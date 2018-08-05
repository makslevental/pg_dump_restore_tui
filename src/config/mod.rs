extern crate toml;
extern crate colored;

use std::fs::File;
use std::io::{self, Read, Write};

mod args;

#[derive(Debug, Deserialize)]
pub struct Config {
    pg_host: String,
    pg_pass: String,
    pg_user: String,
    pg_port: u32,
    psql_bin: String,
    pg_dumpall_bin: String,
}

pub fn load_config() -> Result<Config, String> {
    let args = args::load_args();

    let mut config_file =
        File::open(args.value_of("config_file").unwrap())
            .map_err(|e| format!("config_file: {}", e.to_string()))?;
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str);
    let config_from_file: Config =
        toml::from_str(&config_str)
            .map_err(|e| format!("config_file: {}", e.to_string()))?;

    let config_from_args = Config {
        pg_host: args.value_of("pg_host").unwrap_or_default().to_string(),
        pg_pass: args.value_of("pg_pass").unwrap_or_default().to_string(),
        pg_user: args.value_of("pg_user").unwrap_or_default().to_string(),
        pg_port: value_t!(args, "pg_port", u32).unwrap_or_default(),
        psql_bin: args.value_of("psql_bin").unwrap_or_default().to_string(),
        pg_dumpall_bin: args.value_of("pg_dumpall_bin").unwrap_or_default().to_string(),
    };

    let union_args = Config {
        pg_host: if config_from_args.pg_host.len() > 0 { config_from_args.pg_host } else { config_from_file.pg_host },
        pg_pass: if config_from_args.pg_pass.len() > 0 { config_from_args.pg_pass } else { config_from_file.pg_pass },
        pg_user: if config_from_args.pg_user.len() > 0 { config_from_args.pg_user } else { config_from_file.pg_user },
        pg_port: if config_from_args.pg_port > 0 { config_from_args.pg_port } else { config_from_file.pg_port },
        psql_bin: if config_from_args.psql_bin.len() > 0 { config_from_args.psql_bin } else { config_from_file.psql_bin },
        pg_dumpall_bin: if config_from_args.pg_dumpall_bin.len() > 0 { config_from_args.pg_dumpall_bin } else { config_from_file.pg_dumpall_bin },
    };

    match union_args.pg_host.len() * union_args.pg_pass.len() * union_args.pg_user.len() * union_args.pg_port as usize * union_args.psql_bin.len() * union_args.pg_dumpall_bin.len() {
        0 => {
            let mut missing_args: Vec<&str> = Vec::new();
            if union_args.pg_host.len() == 0 {
                missing_args.push("pg_host");
            };
            if union_args.pg_pass.len() == 0 {
                missing_args.push("pg_pass");
            };
            if union_args.pg_user.len() == 0 {
                missing_args.push("pg_user");
            };
            if union_args.pg_port == 0 {
                missing_args.push("pg_port");
            };
            if union_args.psql_bin.len() == 0 {
                missing_args.push("psql_bin");
            };
            if union_args.pg_dumpall_bin.len() == 0 {
                missing_args.push("pg_dumpall_bin");
            };
            Err(format!("missing args: {}", missing_args.join(", ")))
        },
        _ => Ok(union_args)
    }
}