#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;

// TODO(feature): Docker image output.
// TODO(refactor): Clean up unwrap, other possible panics.

/// Initialise command.
const COMMAND_INIT: &str = "init";

/// Start command.
const COMMAND_START: &str = "start";

/// Service name command line argument.
const ARG_SERVICE_NAME: &str = "NAME";

/// Service URL command line argument.
const ARG_SERVICE_URL: &str = "URL";

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
                        .help("Service url")
                        .required(true)
                        .index(2),
                ]),
            SubCommand::with_name(COMMAND_START)
                .version(crate_version!())
                .about("Start server")
                .author(crate_authors!("\n")),
        ])
        .get_matches();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let server_addr = std::env::var("SERVER_ADDR").unwrap();

    let result = match matches.subcommand() {
        (COMMAND_INIT, Some(submatches)) => {
            let name = submatches.value_of(ARG_SERVICE_NAME).unwrap();
            let url = submatches.value_of(ARG_SERVICE_URL).unwrap();
            ark_auth::cli_init(&db_url, name, url).map(|(service, key)| {
                println!("service_id: {}", service.service_id);
                println!("service_name: {}", service.service_name);
                println!("key_id: {}", key.key_id);
                println!("key_value: {}", key.key_value);
                0
            })
        }
        (COMMAND_START, Some(_submatches)) => match config(server_addr) {
            Ok(config) => ark_auth::cli_start(config, &db_url).map(|_| 0),
            Err(e) => Err(e),
        },
        _ => Err(ark_auth::CliError::InvalidCommand),
    };

    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            error!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn config(server_addr: String) -> Result<ark_auth::api::ApiConfig, ark_auth::CliError> {
    let mut config = ark_auth::api::ApiConfig::new(server_addr);

    let gh_client_id = std::env::var("GITHUB_CLIENT_ID").map_err(ark_auth::CliError::StdEnvVar);
    let gh_client_secret =
        std::env::var("GITHUB_CLIENT_SECRET").map_err(ark_auth::CliError::StdEnvVar);
    let gh_redirect_url =
        std::env::var("GITHUB_REDIRECT_URL").map_err(ark_auth::CliError::StdEnvVar);
    if gh_client_id.is_ok() || gh_client_secret.is_ok() || gh_redirect_url.is_ok() {
        config = config.set_oauth_github(gh_client_id?, gh_client_secret?, gh_redirect_url?);
    }

    let ms_client_id = std::env::var("MICROSOFT_CLIENT_ID").map_err(ark_auth::CliError::StdEnvVar);
    let ms_client_secret =
        std::env::var("MICROSOFT_CLIENT_SECRET").map_err(ark_auth::CliError::StdEnvVar);
    let ms_redirect_url =
        std::env::var("MICROSOFT_REDIRECT_URL").map_err(ark_auth::CliError::StdEnvVar);
    if ms_client_id.is_ok() || ms_client_secret.is_ok() || ms_redirect_url.is_ok() {
        config = config.set_oauth_microsoft(ms_client_id?, ms_client_secret?, ms_redirect_url?);
    }

    Ok(config)
}
