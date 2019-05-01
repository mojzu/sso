#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use ark_auth::driver;
use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;

// TODO(feature): Docker image output.
// TODO(refactor): Clean up unwrap, other possible panics.

const COMMAND_INIT: &str = "init";
const COMMAND_START: &str = "start";

const ARG_SERVICE_NAME: &str = "NAME";
const ARG_SERVICE_URL: &str = "URL";

struct Configuration {
    pub database_url: String,
    pub server_configuration: ark_auth::api::ApiConfig,
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
            SubCommand::with_name(COMMAND_INIT)
                .version(crate_version!())
                .about("Initialise new service")
                .author(crate_authors!("\n"))
                .args(&[
                    Arg::with_name(ARG_SERVICE_NAME)
                        .help("Service name")
                        .required(true)
                        .index(1),
                    Arg::with_name(ARG_SERVICE_URL)
                        .help("Service URL")
                        .required(true)
                        .index(2),
                ]),
            SubCommand::with_name(COMMAND_START)
                .version(crate_version!())
                .about("Start server")
                .author(crate_authors!("\n")),
        ])
        .get_matches();

    // Build configuration from environment and initialise driver.
    let configuration = configuration_from_environment().unwrap();
    let _driver = initialise_driver(&configuration.database_url);

    // Call library functions with command line arguments.
    // TODO(refactor): Library API, postgres/sqlite drivers.
    let result = match matches.subcommand() {
        (COMMAND_INIT, Some(submatches)) => {
            let name = submatches.value_of(ARG_SERVICE_NAME).unwrap();
            let url = submatches.value_of(ARG_SERVICE_URL).unwrap();
            ark_auth::cli_init(&configuration.database_url, name, url).map(|(service, key)| {
                println!("service_id: {}", service.service_id);
                println!("service_name: {}", service.service_name);
                println!("key_id: {}", key.key_id);
                println!("key_value: {}", key.key_value);
                0
            })
        }
        (COMMAND_START, Some(_submatches)) => ark_auth::cli_start(
            configuration.server_configuration,
            &configuration.database_url,
        )
        .map(|_| 0),
        _ => Err(ark_auth::CliError::InvalidCommand),
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
fn configuration_from_environment() -> Result<Configuration, ark_auth::CliError> {
    // TODO(refactor): Clean this up.
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let server_bind = std::env::var("SERVER_BIND").unwrap();
    let mut server_configuration =
        ark_auth::api::ApiConfig::new(server_bind).set_password_pwned(true);

    let smtp_host = std::env::var("SMTP_HOST").map_err(ark_auth::CliError::StdEnvVar);
    let smtp_port = std::env::var("SMTP_PORT").map_err(ark_auth::CliError::StdEnvVar);
    let smtp_user = std::env::var("SMTP_USER").map_err(ark_auth::CliError::StdEnvVar);
    let smtp_password = std::env::var("SMTP_PASSWORD").map_err(ark_auth::CliError::StdEnvVar);
    if smtp_host.is_ok() || smtp_port.is_ok() || smtp_user.is_ok() || smtp_password.is_ok() {
        let smtp_port = smtp_port?.parse::<u16>().unwrap();
        server_configuration =
            server_configuration.set_smtp(smtp_host?, smtp_port, smtp_user?, smtp_password?);
    }

    let gh_client_id = std::env::var("GITHUB_CLIENT_ID").map_err(ark_auth::CliError::StdEnvVar);
    let gh_client_secret =
        std::env::var("GITHUB_CLIENT_SECRET").map_err(ark_auth::CliError::StdEnvVar);
    let gh_redirect_url =
        std::env::var("GITHUB_REDIRECT_URL").map_err(ark_auth::CliError::StdEnvVar);
    if gh_client_id.is_ok() || gh_client_secret.is_ok() || gh_redirect_url.is_ok() {
        server_configuration = server_configuration.set_oauth2_github(
            gh_client_id?,
            gh_client_secret?,
            gh_redirect_url?,
        );
    }

    let ms_client_id = std::env::var("MICROSOFT_CLIENT_ID").map_err(ark_auth::CliError::StdEnvVar);
    let ms_client_secret =
        std::env::var("MICROSOFT_CLIENT_SECRET").map_err(ark_auth::CliError::StdEnvVar);
    let ms_redirect_url =
        std::env::var("MICROSOFT_REDIRECT_URL").map_err(ark_auth::CliError::StdEnvVar);
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
fn initialise_driver(database_url: &str) -> impl driver::Driver {
    driver::postgres::Driver::initialise(database_url).unwrap()
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
fn initialise_driver(database_url: &str) -> impl driver::Driver {
    driver::sqlite::Driver::initialise(database_url).unwrap()
}
