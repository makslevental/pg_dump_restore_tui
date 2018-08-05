use std::fs::File;
use std::io::{self, Read, Write};
use toml;

mod args;

// macro that expands into a bunch of ifs that check if any of the fields are None
macro_rules! zoom_and_enhance {
    ($(#[$struct_meta:meta])*
    pub struct $name:ident { $(pub $fname:ident : $ftype:ty),* }) => {
        $(#[$struct_meta])*
        pub struct $name {
            $(pub $fname: $ftype),*
        }

        impl $name {
            pub fn missing_params(&self) -> Vec<String> {
                let mut missing: Vec<String> = Vec::new();
                // here's the expansion
                $(
                if self.$fname.is_none() {
                    missing.push(stringify!($fname).to_string());
                };
                )*
                missing
            }
        }
    };
}

zoom_and_enhance! {
    #[derive(Debug, Deserialize)]
    pub struct Config {
        pub pg_host: Option<String>,
        pub pg_pass: Option<String>,
        pub pg_user: Option<String>,
        pub pg_port: Option<u32>,
        pub psql_bin: Option<String>,
        pub pg_dumpall_bin: Option<String>
    }
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
        pg_host: args.value_of("pg_host").map(|v| v.to_string()),
        pg_pass: args.value_of("pg_pass").map(|v| v.to_string()),
        pg_user: args.value_of("pg_user").map(|v| v.to_string()),
        pg_port: {
            match value_t!(args, "pg_port", u32) {
                Err(_) => None,
                Ok(v) => Some(v)
            }
        },
        psql_bin: args.value_of("psql_bin").map(|v| v.to_string()),
        pg_dumpall_bin: args.value_of("pg_dumpall_bin").map(|v| v.to_string()),
    };

    let union_args = Config {
        pg_host: config_from_args.pg_host.or(config_from_file.pg_host),
        pg_pass: config_from_args.pg_pass.or(config_from_file.pg_pass),
        pg_user: config_from_args.pg_user.or(config_from_file.pg_user),
        pg_port: config_from_args.pg_port.or(config_from_file.pg_port),
        psql_bin: config_from_args.psql_bin.or(config_from_file.psql_bin),
        pg_dumpall_bin: config_from_args.pg_dumpall_bin.or(config_from_file.pg_dumpall_bin),
    };

    match union_args.missing_params().len() {
        0 => Ok(union_args),
        _ => Err(format!("missing args: {}", union_args.missing_params().join(", ")))
    }
}