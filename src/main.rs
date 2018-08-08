#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate chrono;
extern crate colored;
extern crate cursive;
extern crate regex;
extern crate toml;

use colored::*;

mod config;
mod pg;
mod tui;

static mut CONFIG: Option<config::Config> = None;

fn main() {
    unsafe {
        CONFIG = match config::load_config() {
            Ok(c) => {
                println!("{:#?}", c);
                Some(c)
            }
            Err(e) => panic!("{}: {}", "error".bold().red(), e),
        };
    }

    tui::display()
}
