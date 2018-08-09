#![allow(dead_code)]
#![allow(unused_variables)]
#![feature(try_trait)]
#![allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate colored;
extern crate cursive;
extern crate regex;
extern crate toml;

use colored::*;

mod config;
mod pg;
mod tui;


fn main() {
    // init config
    config::CONFIG.pg_port.unwrap();
    tui::main_display()
}
