use std::process::{Command, exit, ExitStatus};
use std::str;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::fmt::Write as OtherWrite;
use regex::{self, RegexSetBuilder};
use std::error;
use std::fmt;
use std::sync::mpsc::Sender;

use colored::*;
use chrono::prelude::*;

#[derive(Debug)]
pub enum PgError {
    Io(io::Error),
    Regex(regex::Error),
    Postgres(String),
    Utf8(str::Utf8Error),
}

impl fmt::Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PgError::Io(ref err) => write!(f, "IO error: {}", err),
            PgError::Regex(ref err) => write!(f, "Regex error: {}", err),
            PgError::Postgres(ref err) => write!(f, "Postgres error: {}", err),
            PgError::Utf8(ref err) => write!(f, "Utf8 error: {}", err),
        }
    }
}


impl From<io::Error> for PgError {
    fn from(e: io::Error) -> Self {
        PgError::Io(e)
    }
}

impl From<regex::Error> for PgError {
    fn from(e: regex::Error) -> Self {
        PgError::Regex(e)
    }
}

impl From<str::Utf8Error> for PgError {
    fn from(e: str::Utf8Error) -> Self {
        PgError::Utf8(e)
    }
}

pub fn dump(bin: &str, host: &str, user: &str, pass: &str, port: u32, dump_dest_path: &str, dump_prefix: &str) -> Result<(), PgError> {
//    #PGPASSWORD=***REMOVED*** PGOPTIONS='--client-min-messages=warning' psql -v ON_ERROR_STOP=1 --pset pager=off -h localhost -U postgres -f dump.data.sql
    let utc: DateTime<Utc> = Utc::now();
    let dump_file_fp = format!("{}{}.{}.sql", dump_dest_path, dump_prefix, utc.format("%Y_%m_%dT%H_%M_%S").to_string());
    let output = Command::new(&bin)
        .env("PGPASSWORD", &pass)
        .env("PGOPTIONS", "'--client-min-messages=warning'")
        .args(&["-h", &host])
        .arg("-c")
        .args(&["-U", &user])
        .args(&["-p", &port.to_string()])
        .args(&["-f", &dump_file_fp])
        .output()
        .expect("failed to execute dump");


    match output.status.code() {
        Some(0) => clean_dump(dump_dest_path, user),
        _ => match str::from_utf8(&output.stderr) {
            Ok(v) => {
                let command_str = format!(
                    "PGPASSWORD={} PGOPTIONS=--client-min-messages=warning {} -v ON_ERROR_STOP=1 -h {} -c -U {} -p {} -f {}",
                    pass,
                    bin,
                    host,
                    user,
                    port,
                    dump_file_fp);
                Err(PgError::Postgres(format!("\ndump error: {}\ndump command: {}", v, command_str)))
            }
            Err(e) => Err(PgError::Utf8(e))
        }
    }
}

pub fn restore(restore_file_fp: &str, bin: &str, host: &str, user: &str, pass: &str, port: u32) -> Result<(), PgError> {
    let utc: DateTime<Utc> = Utc::now();
    let output = Command::new(bin)
        .env("PGPASSWORD", pass)
        .env("PGOPTIONS", "--client-min-messages=warning")
        .args(&["-v", "ON_ERROR_STOP=1"])
        .args(&["-h", host])
        .args(&["-U", user])
        .args(&["-p", &port.to_string()])
        .args(&["-f", restore_file_fp])
        .output()
        .expect("failed to execute restore");

    match output.status.code() {
        Some(0) => Ok(()),
        _ => match str::from_utf8(&output.stderr) {
            Ok(v) => {
                let command_str = format!(
                    "PGPASSWORD={} PGOPTIONS=--client-min-messages=warning {} -v ON_ERROR_STOP=1 -h {} -U {} -p {} -f {}",
                    pass,
                    bin,
                    host,
                    user,
                    port,
                    restore_file_fp);
                Err(PgError::Postgres(format!("\nrestore error: {}\nrestore command: {}", v, command_str)))
            }
            Err(e) => Err(PgError::Utf8(e))
        }
    }
}

fn clean_dump(dump_file_fp: &str, user: &str) -> Result<(), PgError> {
    let mut file = File::open(&dump_file_fp)?;
    let mut old_data = String::new();
    let mut new_data = String::new();
    file.read_to_string(&mut old_data)?;
    drop(file);

    let set = RegexSetBuilder::new(&[
        &format!(r#"CREATE ROLE {};"#, user),
        &format!(r#"DROP ROLE {};"#, user),
        &format!(r#"ALTER ROLE {} .*;"#, user)
    ]).case_insensitive(true).build()?;

    old_data
        .lines()
        .filter(|line| !set.is_match(line))
        .for_each(|x| write!(new_data, "{}\n", x).unwrap());

    let mut file = File::create(&dump_file_fp)?;
    file.write(new_data.as_bytes())?;
    Ok(())
}