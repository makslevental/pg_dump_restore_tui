use std::process::{Command, exit};
use std::process::ExitStatus;
use std::str;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::fmt::Write as OtherWrite;

use colored::*;
use spinners::{Spinner, Spinners};
use chrono::prelude::*;
use regex::RegexSetBuilder;

pub fn dump(bin: String, host: String, user: String, pass: String, port: u32) {
//    #PGPASSWORD=***REMOVED*** PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f dump.data.sql
    let sp = Spinner::new(Spinners::Dots9, "dumping database".into());
    let utc: DateTime<Utc> = Utc::now();
    let dump_file_fp = format!("tuf_db_postgres_dump.{}.sql", utc.format("%Y_%m_%dT%H_%M_%S").to_string());
    let output = Command::new(bin)
        .env("PGPASSWORD", pass)
        .env("PGOPTIONS", "--client-min-messages=warning")
        .args(&["-h", &host])
        .arg("-c")
        .args(&["-U", &user])
        .args(&["-p", &port.to_string()])
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

pub fn restore(restore_file_fp: String, bin: String, host: String, user: String, pass: String, port: u32) {
    let sp = Spinner::new(Spinners::Dots9, "restoring database".into());
    let utc: DateTime<Utc> = Utc::now();
    let output = Command::new(bin)
        .env("PGPASSWORD", pass)
        .env("PGOPTIONS", "--client-min-messages=warning")
        .args(&["-v", "ON_ERROR_STOP=1"])
        .args(&["-h", &host])
        .args(&["-U", &user])
        .args(&["-p", &port.to_string()])
        .args(&["-f", &restore_file_fp])
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

