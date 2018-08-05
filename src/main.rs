#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
extern crate clap;
extern crate spinners;
extern crate chrono;
extern crate colored;
extern crate regex;

use std::process::{Command, exit};
use std::error::Error;
use std::process::ExitStatus;
use std::str;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::fmt::Write as OtherWrite;

use colored::*;
use spinners::{Spinner, Spinners};
use clap::{Arg, App, ArgMatches};
use chrono::prelude::*;
use regex::RegexSetBuilder;

type Result<T> = std::result::Result<T, Error>;

fn main() {
    let matches = App::new("My Super Program")
        .version("0.01")
        .author("Maksim L <maksim.levental@gmail.com>")
        .about("Ergonomically dumps and restores postgres server")
        .help_short("H")
        .arg(
            Arg::with_name("pg_pass")
                .short("w")
                .long("pg-pass")
                .required(true)
                .value_name("PASS")
                .help("Postgres user password")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("pg_host")
                .short("h")
                .long("pg-host")
                .value_name("HOST")
                .default_value("172.18.0.67")
                .help("Host where postgres lives")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("pg_user")
                .short("u")
                .long("pg-user")
                .default_value("postgres")
                .value_name("USER")
                .help("Postgres user")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("pg_port")
                .short("p")
                .long("pg-port")
                .default_value("6174")
                .value_name("PORT")
                .help("Postgres port")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("pg_dumpall_bin")
                .short("b")
                .long("pg-dumpall-bin")
                .default_value("/usr/bin/pg_dumpall")
                .value_name("BIN_PATH")
                .help("pg_dumpall bin filepath")
                .takes_value(true)
        )
        .get_matches();


    display()
//    dump(
//        matches.value_of("pg_dumpall_bin").unwrap(),
//        matches.value_of("pg_host").unwrap(),
//        matches.value_of("pg_user").unwrap(),
//        matches.value_of("pg_pass").unwrap(),
//        matches.value_of("pg_port").unwrap(),
//    );

//    restore(
//        "tuf_db_postgres_dump.2018_08_05T04_18_18.sql",
//        matches.value_of("pg_host").unwrap(),
//        matches.value_of("pg_user").unwrap(),
//        matches.value_of("pg_pass").unwrap(),
//        matches.value_of("pg_port").unwrap(),
//    )
}

fn dump(bin: &str, host: &str, user: &str, pass: &str, port: &str) {
//    #PGPASSWORD=***REMOVED*** PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f dump.data.sql
    let sp = Spinner::new(Spinners::Dots9, "dumping database".into());
    let utc: DateTime<Utc> = Utc::now();
    let dump_file_fp = format!("tuf_db_postgres_dump.{}.sql", utc.format("%Y_%m_%dT%H_%M_%S").to_string());
    let output = Command::new(bin)
//    let output = Command::new("echo")
        .env("PGPASSWORD", pass)
        .env("PGOPTIONS", "--client-min-messages=warning")
        .args(&["-h", host])
        .arg("-c")
        .args(&["-U", user])
        .args(&["-p", port])
        .args(&["-f", &dump_file_fp])
        .output()
        .expect(&"failed to execute dump".red().bold().to_string());

    sp.stop();

    match output.status.code() {
        Some(0) => {
            let mut file = File::open(&dump_file_fp).unwrap();
            let mut old_data = String::new();
            let mut new_data = String::new();
            file.read_to_string(&mut old_data);
            drop(file);

            let set = RegexSetBuilder::new(&[
                &format!(r#"CREATE ROLE {};"#, user),
                &format!(r#"DROP ROLE {};"#, user),
                &format!(r#"ALTER ROLE {} .*;"#, user)
            ]).case_insensitive(true)
                .build().unwrap();

            old_data
                .lines()
                .filter(|line| !set.is_match(line))
                .for_each(|x| write!(new_data, "{}\n", x).unwrap());

            let mut file = File::create(&dump_file_fp).unwrap();
            file.write(new_data.as_bytes()).unwrap();

            println!("{}", "\nsuccessful dump\n".green().bold().to_string())
        }
        _ => match str::from_utf8(&output.stderr) {
            Ok(v) => eprintln!("\n{} {}\n", "dump error:".red().bold().to_string(), v),
            Err(e) => panic!(format!("Invalid UTF-8 sequence: {}", e).red().bold().to_string())
        }
    }
}

fn restore(restore_file_fp: &str, host: &str, user: &str, pass: &str, port: &str) {
    let sp = Spinner::new(Spinners::Dots9, "restoring database".into());
    let utc: DateTime<Utc> = Utc::now();
    let output = Command::new("psql")
        .env("PGPASSWORD", pass)
        .env("PGOPTIONS", "--client-min-messages=warning")
        .args(&["-v", "ON_ERROR_STOP=1"])
        .args(&["-h", host])
        .args(&["-U", user])
        .args(&["-p", port])
        .args(&["-f", restore_file_fp])
        .output()
        .expect(&"failed to execute restore".red().bold().to_string());

    sp.stop();


    match output.status.code() {
        Some(0) => println!("{}", "\nsuccessful restore\n".green().bold().to_string()),
        _ => match str::from_utf8(&output.stderr) {
            Ok(v) => eprintln!("\n{} {}\n", "restore error:".red().bold().to_string(), v),
            Err(e) => panic!(format!("Invalid UTF-8 sequence: {}", e).red().bold().to_string())
        }
    }
}

extern crate cursive;

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView};
use cursive::Cursive;

// We'll use a SelectView here.
//
// A SelectView is a scrollable list of items, from which the user can select
// one.

fn display() {
    let mut select = SelectView::new().h_align(HAlign::Center);

    // Read the list of cities from separate file, and fill the view with it.
    // (We include the file at compile-time to avoid runtime read errors.)
    let content = include_str!("./assets/cities.txt");
    select.add_all_str(content.lines());

    // Sets the callback for when "Enter" is pressed.
    select.set_on_submit(show_next_window);

    // Let's override the `j` and `k` keys for navigation
    let select = OnEventView::new(select)
        .on_pre_event_inner('k', |s| {
            s.select_up(1);
            Some(EventResult::Consumed(None))
        })
        .on_pre_event_inner('j', |s| {
            s.select_down(1);
            Some(EventResult::Consumed(None))
        });

    let mut siv = Cursive::default();

    // Let's add a BoxView to keep the list at a reasonable size
    // (it can scroll anyway).
    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((20, 10)))
            .title("Where are you from?"),
    );

    siv.run();
}

// Let's put the callback in a separate function to keep it clean,
// but it's not required.
fn show_next_window(siv: &mut Cursive, city: &str) {
    siv.pop_layer();
    let text = format!("{} is a great city!", city);
    siv.add_layer(
        Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()),
    );
}
