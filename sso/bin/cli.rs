//! # sso_cli
//!
//! ## Configuration
//!
//! See [Config](../sso/struct.Config.html).
//!
#[macro_use]
extern crate clap;

use clap::{App, Arg, SubCommand};

const CMD_GENERATE: &str = "generate";
const CMD_SECRET: &str = "secret";
const CMD_PASSWORD: &str = "password";
const CMD_CLIENT: &str = "client";
const CMD_USER: &str = "user";
const CMD_POSTGRES: &str = "postgres";
const CMD_SETUP: &str = "setup";
const CMD_TEARDOWN: &str = "teardown";
const CMD_BACKUP: &str = "backup";
const CMD_AUDIT: &str = "audit";
const CMD_RETENTION: &str = "retention";
const CMD_READ: &str = "read";

const ARG_CONFIG: &str = "config";
const ARG_CLIENT_ID: &str = "client-id";
const ARG_CLIENT_NAME: &str = "client-name";
const ARG_REDIRECT_URI: &str = "redirect-uri";
const ARG_SCOPE: &str = "scope";
const ARG_USER_ID: &str = "user-id";
const ARG_USER_NAME: &str = "user-name";
const ARG_USER_EMAIL: &str = "user-email";
const ARG_ID: &str = "id";
const ARG_DAYS: &str = "days";

#[tokio::main]
async fn main() {
    let matches = App::new("sso-cli")
        .version(crate_version!())
        .about(crate_description!())
        .author("Sam Ward <mail@mojzu.net>")
        .arg(
            Arg::with_name(ARG_CONFIG)
                .long("config")
                .alias("c")
                .help("Configuration file name (without extension)")
                .takes_value(true)
                .required(false),
        )
        .subcommands(vec![
            SubCommand::with_name(CMD_GENERATE)
                .aliases(&["g", "gen"])
                .help("Generate")
                .subcommands(vec![
                    SubCommand::with_name(CMD_SECRET)
                        .alias("s")
                        .help("Generate random secret"),
                    SubCommand::with_name(CMD_PASSWORD)
                        .alias("p")
                        .help("Generate random password and hash"),
                    SubCommand::with_name(CMD_CLIENT)
                        .alias("c")
                        .help("Generate client configuration")
                        .args(&[
                            Arg::with_name(ARG_CLIENT_NAME)
                                .help("Client name")
                                .required(true)
                                .index(1),
                            Arg::with_name(ARG_REDIRECT_URI)
                                .help("Client redirect URI")
                                .required(true)
                                .index(2),
                            Arg::with_name(ARG_CLIENT_ID)
                                .long("client-id")
                                .help("Client ID (UUID)")
                                .takes_value(true)
                                .required(false),
                            Arg::with_name(ARG_SCOPE)
                                .long("scope")
                                .help("Client scope")
                                .takes_value(true)
                                .required(false),
                        ]),
                    SubCommand::with_name(CMD_USER)
                        .alias("u")
                        .help("Generate user configuration")
                        .args(&[
                            Arg::with_name(ARG_USER_NAME)
                                .help("User name")
                                .required(true)
                                .index(1),
                            Arg::with_name(ARG_USER_EMAIL)
                                .help("User email")
                                .required(true)
                                .index(2),
                            Arg::with_name(ARG_USER_ID)
                                .long("user-id")
                                .help("User ID (UUID)")
                                .takes_value(true)
                                .required(false),
                        ]),
                ]),
            SubCommand::with_name(CMD_POSTGRES)
                .aliases(&["p", "pg"])
                .help("Postgres")
                .subcommands(vec![
                    SubCommand::with_name(CMD_SETUP)
                        .alias("s")
                        .help("Setup database"),
                    SubCommand::with_name(CMD_TEARDOWN).help("Teardown database"),
                ]),
            SubCommand::with_name(CMD_BACKUP)
                .aliases(&["b"])
                .help("Create backup"),
            SubCommand::with_name(CMD_AUDIT)
                .aliases(&["a", "aud"])
                .help("Audit")
                .subcommands(vec![
                    SubCommand::with_name(CMD_READ)
                        .alias("r")
                        .help("Read audit log")
                        .arg(
                            Arg::with_name(ARG_ID)
                                .long("id")
                                .help("Audit ID (UUID)")
                                .takes_value(true)
                                .required(false),
                        ),
                    SubCommand::with_name(CMD_RETENTION)
                        .about("Run audit log retention task")
                        .arg(
                            Arg::with_name(ARG_DAYS)
                                .help("Number of days to retain")
                                .required(true)
                                .index(1),
                        ),
                ]),
        ])
        .get_matches();

    let config_name = matches.value_of(ARG_CONFIG).unwrap_or(".config/sso");
    let config =
        sso::config::from_env(config_name).expect("parse configuration from environment failure");
    let config = config
        .load_templates()
        .await
        .expect("load template files failure");

    sso::util::init_panic(config.log.pretty);
    sso::util::init_log(config.log.pretty);

    match matches.subcommand() {
        (CMD_GENERATE, Some(submatches)) => match submatches.subcommand() {
            (CMD_SECRET, Some(_submatches)) => {
                sso::cli::generate_secret(&config).await;
            }
            (CMD_PASSWORD, Some(_submatches)) => {
                sso::cli::generate_password(&config).await;
            }
            (CMD_CLIENT, Some(submatches)) => {
                let client_id = submatches.value_of(ARG_CLIENT_ID);
                let client_name = submatches.value_of(ARG_CLIENT_NAME).unwrap();
                let redirect_uri = submatches.value_of(ARG_REDIRECT_URI).unwrap();
                let scope = submatches.value_of(ARG_SCOPE).unwrap_or("");
                sso::cli::generate_client(&config, client_id, client_name, redirect_uri, scope)
                    .await;
            }
            (CMD_USER, Some(submatches)) => {
                let user_id = submatches.value_of(ARG_USER_ID);
                let user_name = submatches.value_of(ARG_USER_NAME).unwrap();
                let user_email = submatches.value_of(ARG_USER_EMAIL).unwrap();
                sso::cli::generate_user(&config, user_id, user_name, user_email).await;
            }
            _ => {
                println!("{}", submatches.usage());
            }
        },
        (CMD_POSTGRES, Some(submatches)) => match submatches.subcommand() {
            (CMD_SETUP, Some(_submatches)) => {
                sso::cli::postgres_setup(&config).await;
            }
            (CMD_TEARDOWN, Some(_submatches)) => {
                sso::cli::postgres_teardown(&config).await;
            }
            _ => {
                println!("{}", submatches.usage());
            }
        },
        (CMD_BACKUP, Some(_submatches)) => {
            sso::cli::backup(&config).await;
        }
        (CMD_AUDIT, Some(submatches)) => match submatches.subcommand() {
            (CMD_READ, Some(submatches)) => {
                let id = submatches.value_of(ARG_ID);
                let id = id.map(|x| x.parse::<i64>().unwrap());
                sso::cli::audit_read(&config, id).await;
            }
            (CMD_RETENTION, Some(submatches)) => {
                let days = submatches.value_of(ARG_DAYS).unwrap();
                let days = days.parse::<i32>().unwrap();
                sso::cli::audit_retention(&config, days).await;
            }
            _ => {
                println!("{}", submatches.usage());
            }
        },
        _ => {
            println!("{}", matches.usage());
        }
    }
}
