use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView};
use cursive::Cursive;
use std::fs;
use regex::RegexBuilder;

use super::pg;
use super::CONFIG;

pub fn display() {
    let mut select = SelectView::new().h_align(HAlign::Center);

    // Read the list of cities from separate file, and fill the view with it.
    // (We include the file at compile-time to avoid runtime read errors.)
//    let content = include_str!("./assets/cities.txt");
    let paths = fs::read_dir("./").unwrap();

    let reg = RegexBuilder::new(r#"^tuf_db_postgres_dump.*"#).case_insensitive(true).build().unwrap();

    select.add_all_str(
        paths
            .filter(|file| reg.is_match(file.as_ref().unwrap().file_name().to_str().unwrap()))
            .map(|file| file.as_ref().unwrap().file_name().into_string().unwrap())
    );

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
        Dialog::around(select.scrollable().fixed_size((50, 10)))
            .title("Which dump do you want to restore?"),
    );

    siv.run();
}

fn show_next_window(siv: &mut Cursive, tuf_db_dump_file: &str) {
    unsafe {
        match CONFIG {
            None => println!("adsff"),
            Some(ref c) => {
                pg::dump(
                    c.pg_dumpall_bin.clone().unwrap(),
                    c.pg_host.clone().unwrap(),
                    c.pg_user.clone().unwrap(),
                    c.pg_pass.clone().unwrap(),
                    c.pg_port.unwrap()
                )
            }
        }
    }
//    siv.pop_layer();
//    let text = format!("{} is a great city!", city);
//    siv.add_layer(
//        Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()),
//    );
}
