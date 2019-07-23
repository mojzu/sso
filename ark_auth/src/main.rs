#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use ark_auth::{cli, driver, driver::Driver, notify, server};
use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;
use std::env;

const COMMAND_CREATE_ROOT_KEY: &str = "create-root-key";
const COMMAND_DELETE_ROOT_KEYS: &str = "delete-root-keys";
const COMMAND_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const COMMAND_START_SERVER: &str = "start-server";
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
            SubCommand::with_name(COMMAND_CREATE_ROOT_KEY)
                .version(crate_version!())
                .about("Create a root key")
                .author(crate_authors!("\n"))
                .arg(
                    Arg::with_name(ARG_NAME)
                        .help("Key name")
                        .required(true)
                        .index(1),
                ),
            SubCommand::with_name(COMMAND_DELETE_ROOT_KEYS)
                .version(crate_version!())
                .about("Delete all root keys")
                .author(crate_authors!("\n")),
            SubCommand::with_name(COMMAND_CREATE_SERVICE_WITH_KEY)
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
            SubCommand::with_name(COMMAND_START_SERVER)
                .version(crate_version!())
                .about("Start server")
                .author(crate_authors!("\n")),
        ])
        .get_matches();

    // Build configuration and driver from environment.
    let result = configuration_from_environment().and_then(|(configuration, driver)| {
        // Call library functions with command line arguments.
        match matches.subcommand() {
            (COMMAND_CREATE_ROOT_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                cli::create_root_key(driver, name).map(|key| {
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
            }
            (COMMAND_DELETE_ROOT_KEYS, Some(_submatches)) => {
                cli::delete_root_keys(driver).map(|_| 0)
            }
            (COMMAND_CREATE_SERVICE_WITH_KEY, Some(submatches)) => {
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
            (COMMAND_START_SERVER, Some(_submatches)) => {
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

const DATABASE_URL: &str = "DATABASE_URL";
const DATABASE_CONNECTIONS: &str = "DATABASE_CONNECTIONS";
const SERVER_BIND: &str = "SERVER_BIND";
const SMTP_HOST: &str = "SMTP_HOST";
const SMTP_PORT: &str = "SMTP_PORT";
const SMTP_USER: &str = "SMTP_USER";
const SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const GITHUB_CLIENT_ID: &str = "GITHUB_CLIENT_ID";
const GITHUB_CLIENT_SECRET: &str = "GITHUB_CLIENT_SECRET";
const GITHUB_REDIRECT_URL: &str = "GITHUB_REDIRECT_URL";
const MICROSOFT_CLIENT_ID: &str = "MICROSOFT_CLIENT_ID";
const MICROSOFT_CLIENT_SECRET: &str = "MICROSOFT_CLIENT_SECRET";
const MICROSOFT_REDIRECT_URL: &str = "MICROSOFT_REDIRECT_URL";

/// Build configuration and driver from environment.
fn configuration_from_environment() -> Result<(cli::Configuration, Box<Driver>), cli::Error> {
    let database_url = env_database_url()?;
    let database_connections = env_database_connections()?;
    let server_bind = env_server_bind()?;
    let smtp = env_smtp()?;
    let github = env_github()?;
    let microsoft = env_microsoft()?;

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

    Ok((configuration, driver))
}

fn env_database_url() -> Result<String, cli::Error> {
    env::var(DATABASE_URL).map_err(|err| {
        error!("{} is undefined, required ({})", DATABASE_URL, err);
        cli::Error::StdEnvVar(err)
    })
}

fn env_database_connections() -> Result<Option<u32>, cli::Error> {
    let database_connections = env::var(DATABASE_CONNECTIONS).ok();
    if let Some(x) = database_connections {
        match x.parse::<u32>() {
            Ok(x) => Ok(Some(x)),
            Err(err) => {
                error!(
                    "{} is invalid unsigned integer ({})",
                    DATABASE_CONNECTIONS, err
                );
                Err(cli::Error::StdNumParseInt(err))
            }
        }
    } else {
        Ok(None)
    }
}

fn env_server_bind() -> Result<String, cli::Error> {
    env::var(SERVER_BIND).map_err(|err| {
        error!("{} is undefined, required ({})", SERVER_BIND, err);
        cli::Error::StdEnvVar(err)
    })
}

fn env_smtp() -> Result<Option<notify::ConfigurationSmtp>, cli::Error> {
    let smtp_host = std::env::var(SMTP_HOST).map_err(|err| {
        error!("{} is undefined ({})", SMTP_HOST, err);
        cli::Error::StdEnvVar(err)
    });
    let smtp_port = std::env::var(SMTP_PORT).map_err(|err| {
        error!("{} is undefined ({})", SMTP_PORT, err);
        cli::Error::StdEnvVar(err)
    });
    let smtp_user = std::env::var(SMTP_USER).map_err(|err| {
        error!("{} is undefined ({})", SMTP_USER, err);
        cli::Error::StdEnvVar(err)
    });
    let smtp_password = std::env::var(SMTP_PASSWORD).map_err(|err| {
        error!("{} is undefined ({})", SMTP_PASSWORD, err);
        cli::Error::StdEnvVar(err)
    });
    if smtp_host.is_ok() || smtp_port.is_ok() || smtp_user.is_ok() || smtp_password.is_ok() {
        let smtp_host = smtp_host?;
        let smtp_port = smtp_port?;
        let smtp_user = smtp_user?;
        let smtp_password = smtp_password?;

        match smtp_port.parse::<u16>() {
            Ok(x) => Ok(Some(notify::ConfigurationSmtp::new(
                smtp_host,
                x,
                smtp_user,
                smtp_password,
            ))),
            Err(err) => {
                error!("{} is invalid port number ({})", SMTP_PORT, err);
                Err(cli::Error::StdNumParseInt(err))
            }
        }
    } else {
        Ok(None)
    }
}

fn env_github() -> Result<server::ConfigurationProvider, cli::Error> {
    let client_id = std::env::var(GITHUB_CLIENT_ID).map_err(|err| {
        error!("{} is undefined ({})", GITHUB_CLIENT_ID, err);
        cli::Error::StdEnvVar(err)
    });
    let client_secret = std::env::var(GITHUB_CLIENT_SECRET).map_err(|err| {
        error!("{} is undefined ({})", GITHUB_CLIENT_SECRET, err);
        cli::Error::StdEnvVar(err)
    });
    let redirect_url = std::env::var(GITHUB_REDIRECT_URL).map_err(|err| {
        error!("{} is undefined ({})", GITHUB_REDIRECT_URL, err);
        cli::Error::StdEnvVar(err)
    });
    let oauth2 = if client_id.is_ok() || client_secret.is_ok() || redirect_url.is_ok() {
        let client_id = client_id?;
        let client_secret = client_secret?;
        let redirect_url = redirect_url?;
        Some(server::ConfigurationProviderOauth2::new(
            client_id,
            client_secret,
            redirect_url,
        ))
    } else {
        None
    };
    Ok(server::ConfigurationProvider::new(oauth2))
}

fn env_microsoft() -> Result<server::ConfigurationProvider, cli::Error> {
    let client_id = std::env::var(MICROSOFT_CLIENT_ID).map_err(|err| {
        error!("{} is undefined ({})", MICROSOFT_CLIENT_ID, err);
        cli::Error::StdEnvVar(err)
    });
    let client_secret = std::env::var(MICROSOFT_CLIENT_SECRET).map_err(|err| {
        error!("{} is undefined ({})", MICROSOFT_CLIENT_SECRET, err);
        cli::Error::StdEnvVar(err)
    });
    let redirect_url = std::env::var(MICROSOFT_REDIRECT_URL).map_err(|err| {
        error!("{} is undefined ({})", MICROSOFT_REDIRECT_URL, err);
        cli::Error::StdEnvVar(err)
    });
    let oauth2 = if client_id.is_ok() || client_secret.is_ok() || redirect_url.is_ok() {
        let client_id = client_id?;
        let client_secret = client_secret?;
        let redirect_url = redirect_url?;
        Some(server::ConfigurationProviderOauth2::new(
            client_id,
            client_secret,
            redirect_url,
        ))
    } else {
        None
    };
    Ok(server::ConfigurationProvider::new(oauth2))
}
