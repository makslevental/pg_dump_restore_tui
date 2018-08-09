use std::fs;
use regex::{self, RegexBuilder};
use std::{thread, time};
use std::sync::mpsc::{Receiver, channel};
use std::io;
use std::fmt;
use std::error;

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView, Panel};
use cursive::view::SizeConstraint::Fixed;
use cursive::Cursive;
use cursive::utils::Counter;


use super::pg;
use super::CONFIG;
use std::time::Duration;
use std::process::exit;

mod spinner;

#[derive(Debug)]
pub enum TuiError {
    Io(io::Error),
    Regex(regex::Error),

}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TuiError::Io(ref err) => write!(f, "IO error: {}", err),
            TuiError::Regex(ref err) => write!(f, "Regex error: {}", err),
        }
    }
}
//
//impl error::Error for TuiError {
//    fn description(&self) -> &str {
//        match *self {
//            TuiError::Io(ref err) => err.description(),
//            TuiError::Regex(ref err) => err.description(),
//        }
//    }
//}

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

pub fn display() -> Result<(), TuiError> {
    let mut select = SelectView::new().h_align(HAlign::Center);
    // Read the list of cities from separate file, and fill the view with it.
    // (We include the file at compile-time to avoid runtime read errors.)
//    let content = include_str!("./assets/cities.txt");
    let paths = fs::read_dir("./")?;

    let reg = RegexBuilder::new(r#"^*.sql$"#).case_insensitive(true).build()?;

    select.add_all_str(
        paths
            .filter(|file| reg.is_match(file.as_ref().unwrap().file_name().to_str().unwrap()))
            .map(|file| file.as_ref().unwrap().file_name().into_string().unwrap())
    );

    // Sets the callback for when "Enter" is pressed.
    select.set_on_submit(restore);

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

    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((50, 10)))
            .title("which dump do you want to restore?"),
    );

    siv.set_fps(30);
    siv.run();
    Ok(())
}

fn restore(siv: &mut Cursive, tuf_db_dump_file: &str) {
    let cb = siv.cb_sink().clone();
    let fp = tuf_db_dump_file.clone().to_owned();
    siv.add_layer(Dialog::around(
        spinner::Spinner::new()
            // We need to know how many ticks represent a full bar.
            .with_task(move |counter| {
                let res = load(&fp, &counter);
                match res {
                    Ok(_) => cb.send(Box::new(|s: &mut Cursive| success("restore finished", s))),
                    Err(pg_error) => cb.send(Box::new(|s: &mut Cursive| failure(pg_error, s)))
                }
            })
            .fixed_width(20)
    ).title("restoring database"));
}


fn load(restore_file_fp: &str, counter: &Counter) -> Result<(), pg::PgError> {
    let (sender, receiver): (_, Receiver<Result<(), pg::PgError>>) = channel();
    let fp = restore_file_fp.clone().to_owned();
    unsafe {
        thread::spawn(move || {
            match CONFIG {
                None => {
                    eprintln!("how did you get here without loading a config first?");
                    exit(1)
                }
                Some(ref c) => {
                    sender.send(
                        pg::restore(
                            &fp,
                            c.psql_bin.as_ref().unwrap(),
                            c.pg_host.as_ref().unwrap(),
                            c.pg_user.as_ref().unwrap(),
                            c.pg_pass.as_ref().unwrap(),
                            c.pg_port.unwrap(),
                        )
                    )
                }
            }
        });
    }
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


fn failure(pg_error: pg::PgError, s: &mut Cursive) {
    s.pop_layer();
    s.add_layer(
        Dialog::around(
            TextView::new(pg_error.to_string())
        ).h_align(HAlign::Center)
            .title("restore error")
            // This is the alignment for the button
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
            .h_align(HAlign::Center)
            .button("Quit", |s| s.quit()),
    );
}
