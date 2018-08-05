#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate spinners;
extern crate chrono;
extern crate colored;
extern crate regex;
extern crate toml;

use colored::*;
mod config;
mod pg;

fn main() {
    let config = match config::load_config() {
        Ok(c) => {
            println!("{:#?}", c);
            c
        },
        Err(e) => {
            panic!("{}: {}", "error".bold().red(), e)
        }
    };

//    display()
    pg::dump(
        &config.pg_dumpall_bin.clone().unwrap(),
        &config.pg_host.clone().unwrap(),
        &config.pg_user.clone().unwrap(),
        &config.pg_pass.clone().unwrap(),
        config.pg_port.clone().unwrap(),
    );

    pg::restore(
        "tuf_db_postgres_dump.2018_08_05T19_48_40.sql",
        &config.psql_bin.clone().unwrap(),
        &config.pg_host.clone().unwrap(),
        &config.pg_user.clone().unwrap(),
        &config.pg_pass.clone().unwrap(),
        config.pg_port.clone().unwrap(),
    )
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
