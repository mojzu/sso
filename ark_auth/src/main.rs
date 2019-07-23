#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use ark_auth::{cli, driver, driver::Driver, notify, server};
use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;

const ENV_DATABASE_URL: &str = "DATABASE_URL";
const ENV_DATABASE_CONNECTIONS: &str = "DATABASE_CONNECTIONS";
const ENV_SERVER_BIND: &str = "SERVER_BIND";
const ENV_SMTP_HOST: &str = "SMTP_HOST";
const ENV_SMTP_PORT: &str = "SMTP_PORT";
const ENV_SMTP_USER: &str = "SMTP_USER";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_GITHUB_CLIENT_ID: &str = "GITHUB_CLIENT_ID";
const ENV_GITHUB_CLIENT_SECRET: &str = "GITHUB_CLIENT_SECRET";
const ENV_GITHUB_REDIRECT_URL: &str = "GITHUB_REDIRECT_URL";
const ENV_MICROSOFT_CLIENT_ID: &str = "MICROSOFT_CLIENT_ID";
const ENV_MICROSOFT_CLIENT_SECRET: &str = "MICROSOFT_CLIENT_SECRET";
const ENV_MICROSOFT_REDIRECT_URL: &str = "MICROSOFT_REDIRECT_URL";

const CMD_CREATE_ROOT_KEY: &str = "create-root-key";
const CMD_DELETE_ROOT_KEYS: &str = "delete-root-keys";
const CMD_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const CMD_START_SERVER: &str = "start-server";

const ARG_NAME: &str = "NAME";
const ARG_URL: &str = "URL";

fn main() {
    // Configure logging environment variables.
    std::env::set_var("RUST_BACKTRACE", "0");
    std::env::set_var("RUST_LOG", "info");

    // If SENTRY_URL is defined, enable logging and panic handler integration.
    let _guard = match std::env::var("SENTRY_URL") {
        Ok(sentry_url) => {
            let guard = sentry::init(sentry_url);
            let mut options = LoggerOptions::default();
            options.emit_warning_events = true;

            sentry::integrations::env_logger::init(None, options);
            sentry::integrations::panic::register_panic_handler();
            Some(guard)
        }
        Err(e) => {
            env_logger::init();
            warn!("SENTRY_URL is undefined, integration is disabled ({})", e);
            None
        }
    };

    // Command line argument parser.
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
        .subcommands(vec![
            SubCommand::with_name(CMD_CREATE_ROOT_KEY)
                .version(crate_version!())
                .about("Create a root key")
                .author(crate_authors!("\n"))
                .arg(
                    Arg::with_name(ARG_NAME)
                        .help("Key name")
                        .required(true)
                        .index(1),
                ),
            SubCommand::with_name(CMD_DELETE_ROOT_KEYS)
                .version(crate_version!())
                .about("Delete all root keys")
                .author(crate_authors!("\n")),
            SubCommand::with_name(CMD_CREATE_SERVICE_WITH_KEY)
                .version(crate_version!())
                .about("Create service with service key")
                .author(crate_authors!("\n"))
                .args(&[
                    Arg::with_name(ARG_NAME)
                        .help("Service name")
                        .required(true)
                        .index(1),
                    Arg::with_name(ARG_URL)
                        .help("Service URL")
                        .required(true)
                        .index(2),
                ]),
            SubCommand::with_name(CMD_START_SERVER)
                .version(crate_version!())
                .about("Start server")
                .author(crate_authors!("\n")),
        ])
        .get_matches();

    // Build configuration and driver from environment variables.
    let result = configure().and_then(|(driver, configuration)| {
        // Call library functions with command line arguments.
        match matches.subcommand() {
            (CMD_CREATE_ROOT_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                cli::create_root_key(driver, name).map(|key| {
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
            }
            (CMD_DELETE_ROOT_KEYS, Some(_submatches)) => cli::delete_root_keys(driver).map(|_| 0),
            (CMD_CREATE_SERVICE_WITH_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                let url = submatches.value_of(ARG_URL).unwrap();
                cli::create_service_with_key(driver, name, url).map(|(service, key)| {
                    println!("service.id: {}", service.id);
                    println!("service.name: {}", service.name);
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
            }
            (CMD_START_SERVER, Some(_submatches)) => {
                cli::start_server(driver, configuration).map(|_| 0)
            }
            _ => {
                println!("{}", matches.usage());
                Ok(1)
            }
        }
    });

    // Handle errors and exit with code.
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn configure() -> Result<(Box<Driver>, cli::Configuration), cli::Error> {
    let database_url = cli::str_from_env(ENV_DATABASE_URL)?;
    let database_connections = cli::opt_u32_from_env(ENV_DATABASE_CONNECTIONS)?;
    let server_bind = cli::str_from_env(ENV_SERVER_BIND)?;
    let smtp = cli::smtp_from_env(
        ENV_SMTP_HOST,
        ENV_SMTP_PORT,
        ENV_SMTP_USER,
        ENV_SMTP_PASSWORD,
    )?;
    let github = server::ConfigurationProvider::new(cli::oauth2_from_env(
        ENV_GITHUB_CLIENT_ID,
        ENV_GITHUB_CLIENT_SECRET,
        ENV_GITHUB_REDIRECT_URL,
    )?);
    let microsoft = server::ConfigurationProvider::new(cli::oauth2_from_env(
        ENV_MICROSOFT_CLIENT_ID,
        ENV_MICROSOFT_CLIENT_SECRET,
        ENV_MICROSOFT_REDIRECT_URL,
    )?);

    let driver = if database_url.starts_with("postgres") {
        driver::PostgresDriver::initialise(&database_url, database_connections)
            .unwrap()
            .box_clone()
    } else {
        // driver::SqliteDriver::initialise(&database_url, database_connections)
        //     .unwrap()
        //     .box_clone()
        unimplemented!();
    };

    let notify = notify::ConfigurationBuilder::default()
        .smtp(smtp)
        .build()
        .unwrap();
    let server = server::ConfigurationBuilder::default()
        .bind(server_bind)
        .password_pwned_enabled(true)
        .provider(server::ConfigurationProviderGroup::new(github, microsoft))
        .build()
        .unwrap();
    let configuration = cli::Configuration::new(notify, server);

    Ok((driver, configuration))
}
