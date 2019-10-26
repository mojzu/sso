//! Binary application.
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;
use sso::{
    Cli, CliOptions, Client, ClientActorOptions, Driver, DriverPostgres, DriverResult, Env,
    NotifyActorOptionsBuilder, ServerOptionsBuilder, ServerOptionsProvider,
    ServerOptionsProviderGroup,
};

const CRATE_NAME: &str = crate_name!();
const CRATE_VERSION: &str = crate_version!();
const CRATE_DESCRIPTION: &str = crate_description!();
const CRATE_AUTHORS: &str = "Sam Ward <git@mojzu.net>";

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
const ENV_MICROSOFT_CLIENT_ID: &str = "MICROSOFT_CLIENT_ID";
const ENV_MICROSOFT_CLIENT_SECRET: &str = "MICROSOFT_CLIENT_SECRET";

const CMD_CREATE_ROOT_KEY: &str = "create-root-key";
const CMD_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const CMD_START_SERVER: &str = "start-server";

const ARG_NAME: &str = "NAME";
const ARG_URL: &str = "URL";
const ARG_LOCAL_URL: &str = "LOCAL_URL";
const ARG_GITHUB_OAUTH2_URL: &str = "GITHUB_OAUTH2_URL";
const ARG_MICROSOFT_OAUTH2_URL: &str = "MICROSOFT_OAUTH2_URL";

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
    let matches = App::new(CRATE_NAME)
        .version(CRATE_VERSION)
        .about(CRATE_DESCRIPTION)
        .author(CRATE_AUTHORS)
        .subcommands(vec![
            SubCommand::with_name(CMD_CREATE_ROOT_KEY)
                .version(CRATE_VERSION)
                .about("Create a root key")
                .author(CRATE_AUTHORS)
                .arg(
                    Arg::with_name(ARG_NAME)
                        .help("Key name")
                        .required(true)
                        .index(1),
                ),
            SubCommand::with_name(CMD_CREATE_SERVICE_WITH_KEY)
                .version(CRATE_VERSION)
                .about("Create service with service key")
                .author(CRATE_AUTHORS)
                .args(&[
                    Arg::with_name(ARG_NAME)
                        .help("Service name")
                        .required(true)
                        .index(1),
                    Arg::with_name(ARG_URL)
                        .help("Service URL")
                        .required(true)
                        .index(2),
                    Arg::with_name(ARG_LOCAL_URL)
                        .long("local-url")
                        .help("Local provider callback URL")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name(ARG_GITHUB_OAUTH2_URL)
                        .long("github-oauth2-url")
                        .help("GitHub OAuth2 provider callback URL")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name(ARG_MICROSOFT_OAUTH2_URL)
                        .long("microsoft-oauth2-url")
                        .help("Microsoft OAuth2 provider callback URL")
                        .takes_value(true)
                        .required(false),
                ]),
            SubCommand::with_name(CMD_START_SERVER)
                .version(CRATE_VERSION)
                .about("Start server")
                .author(CRATE_AUTHORS),
        ])
        .get_matches();

    // Build options and driver from environment variables.
    let result = configure().and_then(|(driver, options)| {
        // Call library functions with command line arguments.
        match matches.subcommand() {
            (CMD_CREATE_ROOT_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                Cli::create_root_key(driver, name).map(|key| {
                    println!("{}", key);
                    0
                })
            }
            (CMD_CREATE_SERVICE_WITH_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                let url = submatches.value_of(ARG_URL).unwrap();
                let provider_local_url = submatches.value_of(ARG_LOCAL_URL);
                let provider_github_oauth2_url = submatches.value_of(ARG_GITHUB_OAUTH2_URL);
                let provider_microsoft_oauth2_url = submatches.value_of(ARG_MICROSOFT_OAUTH2_URL);
                Cli::create_service_with_key(
                    driver,
                    name,
                    url,
                    provider_local_url,
                    provider_github_oauth2_url,
                    provider_microsoft_oauth2_url,
                )
                .map(|(service, key)| {
                    println!("{}", service);
                    println!("{}", key);
                    0
                })
            }
            (CMD_START_SERVER, Some(_submatches)) => Cli::start_server(driver, options).map(|_| 0),
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

fn configure() -> DriverResult<(Box<dyn Driver>, CliOptions)> {
    let database_url = Env::string(ENV_DATABASE_URL)?;
    let database_connections = Env::value_opt::<u32>(ENV_DATABASE_CONNECTIONS)?;
    let server_hostname = Env::string_opt(ENV_SERVER_HOSTNAME).unwrap_or_else(|| "sso".to_owned());
    let server_bind = Env::string(ENV_SERVER_BIND)?;
    let smtp = Env::smtp(
        ENV_SMTP_HOST,
        ENV_SMTP_PORT,
        ENV_SMTP_USER,
        ENV_SMTP_PASSWORD,
    )?;
    let password_pwned_enabled =
        Env::value_opt::<bool>(ENV_PASSWORD_PWNED_ENABLED)?.unwrap_or(false);
    let github =
        ServerOptionsProvider::new(Env::oauth2(ENV_GITHUB_CLIENT_ID, ENV_GITHUB_CLIENT_SECRET)?);
    let microsoft = ServerOptionsProvider::new(Env::oauth2(
        ENV_MICROSOFT_CLIENT_ID,
        ENV_MICROSOFT_CLIENT_SECRET,
    )?);
    let rustls = Env::rustls(
        ENV_SERVER_TLS_CRT_PEM,
        ENV_SERVER_TLS_KEY_PEM,
        ENV_SERVER_TLS_CLIENT_PEM,
    )?;

    let driver = if database_url.starts_with("postgres") {
        DriverPostgres::initialise(&database_url, database_connections)
            .unwrap()
            .box_clone()
    } else {
        // DriverSqlite::initialise(&database_url, database_connections)
        //     .unwrap()
        //     .box_clone()
        unimplemented!();
    };

    let client = ClientActorOptions::new(Client::default_user_agent(), None, None).unwrap();
    let notify = NotifyActorOptionsBuilder::default()
        .smtp(smtp)
        .build()
        .unwrap();
    let server = ServerOptionsBuilder::default()
        .hostname(server_hostname)
        .bind(server_bind)
        .password_pwned_enabled(password_pwned_enabled)
        .provider(ServerOptionsProviderGroup::new(github, microsoft))
        .rustls(rustls)
        .user_agent(Client::default_user_agent())
        .build()
        .unwrap();
    let options = CliOptions::new(client, 2, notify, 4, server);

    Ok((driver, options))
}
