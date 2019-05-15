#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use ark_auth::{cli, core, driver, driver::Driver, server};
use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;

// TODO(feature): Docker image output.

const COMMAND_CREATE_ROOT_KEY: &str = "create-root-key";
const COMMAND_DELETE_ROOT_KEYS: &str = "delete-root-keys";
const COMMAND_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const COMMAND_START_SERVER: &str = "start-server";
const ARG_NAME: &str = "NAME";
const ARG_URL: &str = "URL";

/// Main errors.
#[derive(Debug, Fail)]
enum Error {
    /// Command invalid.
    #[fail(display = "MainError::CommandInvalid")]
    CommandInvalid,
    /// Core error wrapper.
    #[fail(display = "MainError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Server error wrapper.
    #[fail(display = "MainError::Server {}", _0)]
    Server(#[fail(cause)] server::Error),
    /// Standard environment variable error wrapper.
    #[fail(display = "MainError::StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),
}

struct Configuration {
    database_url: String,
    server_configuration: server::Configuration,
}

fn main() {
    // Configure logging environment variables.
    std::env::set_var("RUST_BACKTRACE", "1");
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

    // Build configuration from environment and initialise driver.
    let configuration = configuration_from_environment().unwrap();
    let driver = initialise_driver(&configuration.database_url).box_clone();

    // Call library functions with command line arguments.
    let result = match matches.subcommand() {
        (COMMAND_CREATE_ROOT_KEY, Some(submatches)) => {
            let name = submatches.value_of(ARG_NAME).unwrap();
            cli::create_root_key(driver, name)
                .map_err(Error::Core)
                .map(|key| {
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
        }
        (COMMAND_DELETE_ROOT_KEYS, Some(_submatches)) => cli::delete_root_keys(driver)
            .map_err(Error::Core)
            .map(|_| 0),
        (COMMAND_CREATE_SERVICE_WITH_KEY, Some(submatches)) => {
            let name = submatches.value_of(ARG_NAME).unwrap();
            let url = submatches.value_of(ARG_URL).unwrap();
            cli::create_service_with_key(driver, name, url)
                .map_err(Error::Core)
                .map(|(service, key)| {
                    println!("service.id: {}", service.id);
                    println!("service.name: {}", service.name);
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
        }
        (COMMAND_START_SERVER, Some(_submatches)) => {
            cli::start_server(driver, configuration.server_configuration)
                .map_err(Error::Server)
                .map(|_| 0)
        }
        _ => Err(Error::CommandInvalid),
    };

    // Handle errors and exit with code.
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Build configuration from environment.
fn configuration_from_environment() -> Result<Configuration, Error> {
    // TODO(refactor): Clean this up.
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let server_bind = std::env::var("SERVER_BIND").unwrap();
    let mut server_configuration =
        server::Configuration::new(server_bind).set_password_pwned_enabled(true);

    let smtp_host = std::env::var("SMTP_HOST").map_err(Error::StdEnvVar);
    let smtp_port = std::env::var("SMTP_PORT").map_err(Error::StdEnvVar);
    let smtp_user = std::env::var("SMTP_USER").map_err(Error::StdEnvVar);
    let smtp_password = std::env::var("SMTP_PASSWORD").map_err(Error::StdEnvVar);
    if smtp_host.is_ok() || smtp_port.is_ok() || smtp_user.is_ok() || smtp_password.is_ok() {
        let smtp_port = smtp_port?.parse::<u16>().unwrap();
        server_configuration =
            server_configuration.set_smtp(smtp_host?, smtp_port, smtp_user?, smtp_password?);
    }

    let gh_client_id = std::env::var("GITHUB_CLIENT_ID").map_err(Error::StdEnvVar);
    let gh_client_secret = std::env::var("GITHUB_CLIENT_SECRET").map_err(Error::StdEnvVar);
    let gh_redirect_url = std::env::var("GITHUB_REDIRECT_URL").map_err(Error::StdEnvVar);
    if gh_client_id.is_ok() || gh_client_secret.is_ok() || gh_redirect_url.is_ok() {
        server_configuration = server_configuration.set_oauth2_github(
            gh_client_id?,
            gh_client_secret?,
            gh_redirect_url?,
        );
    }

    let ms_client_id = std::env::var("MICROSOFT_CLIENT_ID").map_err(Error::StdEnvVar);
    let ms_client_secret = std::env::var("MICROSOFT_CLIENT_SECRET").map_err(Error::StdEnvVar);
    let ms_redirect_url = std::env::var("MICROSOFT_REDIRECT_URL").map_err(Error::StdEnvVar);
    if ms_client_id.is_ok() || ms_client_secret.is_ok() || ms_redirect_url.is_ok() {
        server_configuration = server_configuration.set_oauth2_microsoft(
            ms_client_id?,
            ms_client_secret?,
            ms_redirect_url?,
        );
    }

    Ok(Configuration {
        database_url,
        server_configuration,
    })
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
fn initialise_driver(database_url: &str) -> impl Driver {
    // TODO(feature): Configurable number of connections.
    driver::postgres::Driver::initialise(database_url, 10).unwrap()
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
fn initialise_driver(database_url: &str) -> impl Driver {
    driver::sqlite::Driver::initialise(database_url).unwrap()
}
