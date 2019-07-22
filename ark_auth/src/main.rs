#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use ark_auth::{cli, driver, driver::Driver};
use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;

const COMMAND_CREATE_ROOT_KEY: &str = "create-root-key";
const COMMAND_DELETE_ROOT_KEYS: &str = "delete-root-keys";
const COMMAND_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const COMMAND_START_SERVER: &str = "start-server";
const ARG_NAME: &str = "NAME";
const ARG_URL: &str = "URL";

struct Configuration {
    database_url: String,
    configuration: cli::Configuration,
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

    // Build configuration and driver from environment.
    let (configuration, driver) = configuration_from_environment();

    // Call library functions with command line arguments.
    let result = match matches.subcommand() {
        (COMMAND_CREATE_ROOT_KEY, Some(submatches)) => {
            let name = submatches.value_of(ARG_NAME).unwrap();
            cli::create_root_key(driver, name).map(|key| {
                println!("key.id: {}", key.id);
                println!("key.value: {}", key.value);
                0
            })
        }
        (COMMAND_DELETE_ROOT_KEYS, Some(_submatches)) => cli::delete_root_keys(driver).map(|_| 0),
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
            cli::start_server(driver, configuration.configuration).map(|_| 0)
        }
        _ => {
            println!("{}", matches.usage());
            Ok(1)
        }
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
fn configuration_from_environment() -> (Configuration, Box<Driver>) {
    // TODO(refactor): Clean this up, improve error messages.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is undefined, required");
    let database_connections =
        std::env::var("DATABASE_CONNECTIONS").unwrap_or_else(|_| "10".to_owned());
    let database_connections = database_connections.parse::<u32>().unwrap();
    let server_bind = std::env::var("SERVER_BIND").expect("SERVER_BIND is undefined, required");
    let mut configuration = cli::Configuration::new(server_bind);

    let smtp_host = std::env::var("SMTP_HOST");
    let smtp_port = std::env::var("SMTP_PORT");
    let smtp_user = std::env::var("SMTP_USER");
    let smtp_password = std::env::var("SMTP_PASSWORD");
    if smtp_host.is_ok() || smtp_port.is_ok() || smtp_user.is_ok() || smtp_password.is_ok() {
        let smtp_port = smtp_port.unwrap().parse::<u16>().unwrap();
        configuration.mut_notify().set_smtp(
            smtp_host.unwrap(),
            smtp_port,
            smtp_user.unwrap(),
            smtp_password.unwrap(),
        );
    }

    configuration.mut_server().set_password_pwned_enabled(true);

    let gh_client_id = std::env::var("GITHUB_CLIENT_ID");
    let gh_client_secret = std::env::var("GITHUB_CLIENT_SECRET");
    let gh_redirect_url = std::env::var("GITHUB_REDIRECT_URL");
    if gh_client_id.is_ok() || gh_client_secret.is_ok() || gh_redirect_url.is_ok() {
        configuration.mut_server().set_provider_github_oauth2(
            gh_client_id.unwrap(),
            gh_client_secret.unwrap(),
            gh_redirect_url.unwrap(),
        );
    }

    let ms_client_id = std::env::var("MICROSOFT_CLIENT_ID");
    let ms_client_secret = std::env::var("MICROSOFT_CLIENT_SECRET");
    let ms_redirect_url = std::env::var("MICROSOFT_REDIRECT_URL");
    if ms_client_id.is_ok() || ms_client_secret.is_ok() || ms_redirect_url.is_ok() {
        configuration.mut_server().set_provider_microsoft_oauth2(
            ms_client_id.unwrap(),
            ms_client_secret.unwrap(),
            ms_redirect_url.unwrap(),
        );
    }

    let configuration = Configuration {
        database_url,
        configuration,
    };

    let driver = if configuration.database_url.starts_with("postgres") {
        driver::PostgresDriver::initialise(&configuration.database_url, database_connections)
            .unwrap()
            .box_clone()
    } else {
        // driver::SqliteDriver::initialise(&configuration.database_url, database_connections)
        //     .unwrap()
        //     .box_clone()
        unimplemented!();
    };

    (configuration, driver)
}
