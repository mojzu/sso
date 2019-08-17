#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use ark_auth::cli::*;
use ark_auth::{client, driver, driver::Driver, notify, server};
use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;

const ENV_SENTRY_URL: &str = "SENTRY_URL";
const ENV_DATABASE_URL: &str = "DATABASE_URL";
const ENV_DATABASE_CONNECTIONS: &str = "DATABASE_CONNECTIONS";
const ENV_SERVER_HOSTNAME: &str = "SERVER_HOSTNAME";
const ENV_SERVER_BIND: &str = "SERVER_BIND";
const ENV_SERVER_TLS_CRT_PEM: &str = "SERVER_TLS_CRT_PEM";
const ENV_SERVER_TLS_KEY_PEM: &str = "SERVER_TLS_KEY_PEM";
const ENV_SERVER_TLS_CLIENT_PEM: &str = "SERVER_TLS_CLIENT_PEM";
const ENV_SMTP_HOST: &str = "SMTP_HOST";
const ENV_SMTP_PORT: &str = "SMTP_PORT";
const ENV_SMTP_USER: &str = "SMTP_USER";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_PASSWORD_PWNED_ENABLED: &str = "PASSWORD_PWNED_ENABLED";
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
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "info");

    // If SENTRY_URL is defined, enable logging and panic handler integration.
    let _guard = match std::env::var(ENV_SENTRY_URL) {
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

    // Build options and driver from environment variables.
    let result = configure().and_then(|(driver, options)| {
        // Call library functions with command line arguments.
        match matches.subcommand() {
            (CMD_CREATE_ROOT_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                create_root_key(driver, name).map(|key| {
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
            }
            (CMD_DELETE_ROOT_KEYS, Some(_submatches)) => delete_root_keys(driver).map(|_| 0),
            (CMD_CREATE_SERVICE_WITH_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                let url = submatches.value_of(ARG_URL).unwrap();
                create_service_with_key(driver, name, url).map(|(service, key)| {
                    println!("service.id: {}", service.id);
                    println!("service.name: {}", service.name);
                    println!("key.id: {}", key.id);
                    println!("key.value: {}", key.value);
                    0
                })
            }
            (CMD_START_SERVER, Some(_submatches)) => start_server(driver, options).map(|_| 0),
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

fn configure() -> Result<(Box<dyn Driver>, CliOptions), CliError> {
    let database_url = env_string(ENV_DATABASE_URL)?;
    let database_connections = env_value_opt::<u32>(ENV_DATABASE_CONNECTIONS)?;
    let server_hostname =
        env_string_opt(ENV_SERVER_HOSTNAME).unwrap_or_else(|| "ark_auth".to_owned());
    let server_bind = env_string(ENV_SERVER_BIND)?;
    let smtp = env_smtp(
        ENV_SMTP_HOST,
        ENV_SMTP_PORT,
        ENV_SMTP_USER,
        ENV_SMTP_PASSWORD,
    )?;
    let password_pwned_enabled =
        env_value_opt::<bool>(ENV_PASSWORD_PWNED_ENABLED)?.unwrap_or(false);
    let github = server::ServerOptionsProvider::new(env_oauth2(
        ENV_GITHUB_CLIENT_ID,
        ENV_GITHUB_CLIENT_SECRET,
        ENV_GITHUB_REDIRECT_URL,
    )?);
    let microsoft = server::ServerOptionsProvider::new(env_oauth2(
        ENV_MICROSOFT_CLIENT_ID,
        ENV_MICROSOFT_CLIENT_SECRET,
        ENV_MICROSOFT_REDIRECT_URL,
    )?);
    let rustls = env_rustls(
        ENV_SERVER_TLS_CRT_PEM,
        ENV_SERVER_TLS_KEY_PEM,
        ENV_SERVER_TLS_CLIENT_PEM,
    )?;

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

    let client =
        client::ClientExecutorOptions::new(client::default_user_agent(), None, None).unwrap();
    let notify = notify::NotifyExecutorOptionsBuilder::default()
        .smtp(smtp)
        .build()
        .unwrap();
    let server = server::ServerOptionsBuilder::default()
        .hostname(server_hostname)
        .bind(server_bind)
        .password_pwned_enabled(password_pwned_enabled)
        .provider(server::ServerOptionsProviderGroup::new(github, microsoft))
        .rustls(rustls)
        .build()
        .unwrap();
    let options = CliOptions::new(client, 2, notify, 4, server);

    Ok((driver, options))
}
