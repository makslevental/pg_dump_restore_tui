use std::fs;
use regex::{self, RegexBuilder};
use std::{thread, time};
use std::sync::mpsc::{Receiver, channel};
use std::io;
use std::fmt;
use std::error;
use std::option::NoneError;

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView, Panel};
use cursive::view::SizeConstraint::Fixed;
use cursive::Cursive;
use cursive::utils::Counter;


use super::pg;
use super::config::CONFIG;
use std::time::Duration;
use std::process::exit;

mod spinner;

#[derive(Debug)]
pub enum TuiError {
    Io(io::Error),
    Regex(regex::Error),
    Config(NoneError),
}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TuiError::Io(ref err) => write!(f, "IO error: {}", err),
            TuiError::Regex(ref err) => write!(f, "Regex error: {}", err),
            TuiError::Config(ref err) => write!(f, "Missing arg (which is unknown)"),
        }
    }
}

impl From<io::Error> for TuiError {
    fn from(e: io::Error) -> Self {
        TuiError::Io(e)
    }
}

impl From<regex::Error> for TuiError {
    fn from(e: regex::Error) -> Self {
        TuiError::Regex(e)
    }
}

impl From<NoneError> for TuiError {
    fn from(e: NoneError) -> Self {
        TuiError::Config(e)
    }
}

pub fn main_display() {
    let mut siv = Cursive::default();
    siv.set_fps(30);
    siv.add_layer(
        // Most views can be configured in a chainable way
        Dialog::around(TextView::new("which would you like to do?"))
            .title("pg tui")
            .button("Restore", |s| restore_choices_display(s))
            .button("Dump", |s| dump(s))
            .h_align(HAlign::Center)
            .button("Quit", |s| s.quit())
    );
    siv.run();
}


fn restore_choices_display(siv: &mut Cursive) {
    let mut select = SelectView::new().h_align(HAlign::Center);
    let paths = fs::read_dir(CONFIG.dump_loc_path.as_ref().unwrap()).unwrap();
    let reg = RegexBuilder::new(r#"^*.sql$"#).case_insensitive(true).build().unwrap();
    select.add_all_str(
        paths
            .filter(|file| reg.is_match(file.as_ref().unwrap().file_name().to_str().unwrap()))
            .map(|file| file.as_ref().unwrap().file_name().into_string().unwrap())
    );

    select.set_on_submit(|siv: &mut Cursive, db_dump_file: &str| {
        let cb = siv.cb_sink().clone();
        let fp = db_dump_file.clone().to_owned();
        siv.add_layer(Dialog::around(
            spinner::Spinner::new()
                // We need to know how many ticks represent a full bar.
                .with_task(move |counter| {
                    let res = restore(&fp, &counter);
                    match res {
                        Ok(_) => cb.send(Box::new(|s: &mut Cursive| success("restore finished", s))),
                        Err(pg_error) => cb.send(Box::new(|s: &mut Cursive| failure(pg_error, s)))
                    }
                })
                .fixed_width(20)
        ).title("restoring database"));
    });

    siv.add_layer(
        Dialog::around(
            select.scrollable().fixed_size((50, 10))
        )
            .title("which dump do you want to restore?")
            .button("Back", |s| { s.pop_layer(); })
            .h_align(HAlign::Center)
    );
}

fn restore(dump_file_selection: &str, counter: &Counter) -> Result<(), pg::PgError> {
    let (sender, receiver): (_, Receiver<Result<(), pg::PgError>>) = channel();
    let fp = format!("{}{}", CONFIG.dump_loc_path.as_ref().unwrap(), dump_file_selection);
    thread::spawn(move || {
        sender.send(
            pg::restore(
                &fp,
                CONFIG.psql_bin.as_ref().unwrap(),
                CONFIG.pg_host.as_ref().unwrap(),
                CONFIG.pg_user.as_ref().unwrap(),
                CONFIG.pg_pass.as_ref().unwrap(),
                CONFIG.pg_port.unwrap(),
            )
        )
    });
    loop {
        thread::sleep(Duration::from_millis(50));
        counter.tick(1);
        let message = receiver.try_recv();
        match message {
            Ok(r) => return r,
            Err(_) => continue,
        };
    }
}

fn dump(siv: &mut Cursive) {
    let cb = siv.cb_sink().clone();
    siv.add_layer(Dialog::around(
        spinner::Spinner::new()
            // We need to know how many ticks represent a full bar.
            .with_task(move |counter| {
                let (sender, receiver): (_, Receiver<Result<(), pg::PgError>>) = channel();
                thread::spawn(move || {
                    sender.send(
                        pg::dump(
                            CONFIG.pg_dumpall_bin.as_ref().unwrap(),
                            CONFIG.pg_host.as_ref().unwrap(),
                            CONFIG.pg_user.as_ref().unwrap(),
                            CONFIG.pg_pass.as_ref().unwrap(),
                            CONFIG.pg_port.unwrap(),
                            CONFIG.dump_dest_path.as_ref().unwrap(),
                            CONFIG.dump_name_prefix.as_ref().unwrap(),
                        )
                    )
                });

                loop {
                    thread::sleep(Duration::from_millis(50));
                    counter.tick(1);
                    let message = receiver.try_recv();
                    match message {
                        Ok(res) => match res {
                            Ok(_) => cb.send(Box::new(|s: &mut Cursive| success("dump finished", s))),
                            Err(pg_error) => cb.send(Box::new(|s: &mut Cursive| failure(pg_error, s)))
                        },
                        Err(_) => continue,
                    }
                }
            })
            .fixed_width(20)
    ).title("dumping database"));
}

fn failure(pg_error: pg::PgError, s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::around(
            TextView::new(pg_error.to_string()).fixed_width(200)
        ).h_align(HAlign::Center)
            .title("restore error")
            .h_align(HAlign::Left)
            .button("Back", |s| { s.pop_layer(); })
            .h_align(HAlign::Center)
            .button("Quit", |s| s.quit()),
    );
}

fn success(msg: &str, s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .h_align(HAlign::Center)
            .title("job done")
            .content(TextView::new(msg).center())
            .h_align(HAlign::Left)
            .button("Back", |s| { s.pop_layer(); })
            .h_align(HAlign::Center)
            .button("Quit", |s| s.quit()),
    );
}
