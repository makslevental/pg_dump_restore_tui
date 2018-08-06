use std::fs;
use regex::RegexBuilder;
use std::{thread, time};

use cursive::align::HAlign;
use cursive::event::EventResult;
use cursive::traits::*;
use cursive::views::{Dialog, OnEventView, SelectView, TextView, ProgressBar};
use cursive::Cursive;
use cursive::utils::Counter;


use super::pg;
use super::CONFIG;
use std::time::Duration;

mod spinner;

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

    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((50, 10)))
            .title("Which dump do you want to restore?"),
    );

    siv.set_fps(30);
    siv.run();
}

fn show_next_window(siv: &mut Cursive, tuf_db_dump_file: &str) {
    let ten_millis = time::Duration::from_millis(10);
    let cb = siv.cb_sink().clone();
    let n_max = 100000;

    siv.add_layer(Dialog::around(
        ProgressBar::new()
            // We need to know how many ticks represent a full bar.
            .range(0, n_max)
            .with_task(move |counter| {
                fake_load(n_max, &counter);
                cb.send(Box::new(coffee_break));
            })
            .full_width(),
    ));
//    unsafe {
//        match CONFIG {
//            None => println!("adsff"),
//            Some(ref c) => {
//                pg::dump(
//                    c.pg_dumpall_bin.clone().unwrap(),
//                    c.pg_host.clone().unwrap(),
//                    c.pg_user.clone().unwrap(),
//                    c.pg_pass.clone().unwrap(),
//                    c.pg_port.unwrap(),
//                )
//            }
//        }
//    }
//    siv.pop_layer();
//    siv.pop_layer();
//    let text = format!("{} is a great city!", city);
//    siv.add_layer(
//        Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()),
//    );
}


fn fake_load(n_max: usize, counter: &Counter) {
    for _ in 0..n_max {
        thread::sleep(Duration::from_millis(5));
        // The `counter.tick()` method increases the progress value
        counter.tick(1);
    }
}


fn coffee_break(s: &mut Cursive) {
    // A little break before things get serious.
    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title("Preparation complete")
            .content(TextView::new("Now, the real deal!").center())
    );
}
