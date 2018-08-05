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
extern crate cursive;

use colored::*;

mod config;
mod tui;
mod pg;


static mut CONFIG: Option<config::Config> = None;

fn main() {

    unsafe {
        CONFIG = match config::load_config() {
            Ok(c) => {
                println!("{:#?}", c);
                Some(c)
            },
            Err(e) => {
                panic!("{}: {}", "error".bold().red(), e)
            }
        };
    }

    tui::display()
//    pg::dump(
//        &config.pg_dumpall_bin.clone().unwrap(),
//        &config.pg_host.clone().unwrap(),
//        &config.pg_user.clone().unwrap(),
//        &config.pg_pass.clone().unwrap(),
//        config.pg_port.clone().unwrap(),
//    );
//
//    pg::restore(
//        "tuf_db_postgres_dump.2018_08_05T19_48_40.sql",
//        &config.psql_bin.clone().unwrap(),
//        &config.pg_host.clone().unwrap(),
//        &config.pg_user.clone().unwrap(),
//        &config.pg_pass.clone().unwrap(),
//        config.pg_port.clone().unwrap(),
//    )
}


