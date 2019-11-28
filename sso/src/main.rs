//! Binary application.
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;
use sso::{
    env, Cli, CliOptions, Driver, DriverPostgres, DriverResult, ServerOptions,
    ServerOptionsProvider, ServerOptionsProviderGroup,
};

const CRATE_NAME: &str = crate_name!();
const CRATE_VERSION: &str = crate_version!();
const CRATE_DESCRIPTION: &str = crate_description!();
const CRATE_AUTHORS: &str = "Sam Ward <git@mojzu.net>";

const ENV_SENTRY_URL: &str = "SENTRY_URL";
const ENV_DATABASE_URL: &str = "DATABASE_URL";
const ENV_DATABASE_CONNECTIONS: &str = "DATABASE_CONNECTIONS";
const ENV_SERVER_THREADS: &str = "SERVER_THREADS";
const ENV_SERVER_HOSTNAME: &str = "SERVER_HOSTNAME";
const ENV_SERVER_BIND: &str = "SERVER_BIND";
const ENV_SERVER_TLS_CRT_PEM: &str = "SERVER_TLS_CRT_PEM";
const ENV_SERVER_TLS_KEY_PEM: &str = "SERVER_TLS_KEY_PEM";
const ENV_SERVER_TLS_CLIENT_PEM: &str = "SERVER_TLS_CLIENT_PEM";
const ENV_SMTP_HOST: &str = "SMTP_HOST";
const ENV_SMTP_PORT: &str = "SMTP_PORT";
const ENV_SMTP_USER: &str = "SMTP_USER";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_SMTP_FILE: &str = "SMTP_FILE";
const ENV_PASSWORD_PWNED_ENABLED: &str = "PASSWORD_PWNED_ENABLED";
const ENV_GITHUB_CLIENT_ID: &str = "GITHUB_CLIENT_ID";
const ENV_GITHUB_CLIENT_SECRET: &str = "GITHUB_CLIENT_SECRET";
const ENV_MICROSOFT_CLIENT_ID: &str = "MICROSOFT_CLIENT_ID";
const ENV_MICROSOFT_CLIENT_SECRET: &str = "MICROSOFT_CLIENT_SECRET";

const CMD_CREATE_ROOT_KEY: &str = "create-root-key";
const CMD_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const CMD_START_SERVER: &str = "start-server";
const CMD_START_SERVER2: &str = "start-server2";

const ARG_NAME: &str = "NAME";
const ARG_URL: &str = "URL";
const ARG_ALLOW_REGISTER: &str = "ALLOW_REGISTER";
const ARG_EMAIL_TEXT: &str = "EMAIL_TEXT";
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
                    Arg::with_name(ARG_ALLOW_REGISTER)
                        .long("allow-register")
                        .help("Allow user registration")
                        .takes_value(true)
                        .required(false),
                    Arg::with_name(ARG_EMAIL_TEXT)
                        .long("email-text")
                        .help("Text append to user emails")
                        .takes_value(true)
                        .required(false),
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
            SubCommand::with_name(CMD_START_SERVER2)
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
                let user_allow_register = submatches.value_of(ARG_ALLOW_REGISTER);
                let user_email_text = submatches.value_of(ARG_EMAIL_TEXT);
                let provider_local_url = submatches.value_of(ARG_LOCAL_URL);
                let provider_github_oauth2_url = submatches.value_of(ARG_GITHUB_OAUTH2_URL);
                let provider_microsoft_oauth2_url = submatches.value_of(ARG_MICROSOFT_OAUTH2_URL);
                Cli::create_service_with_key(
                    driver,
                    name,
                    url,
                    user_allow_register,
                    user_email_text,
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
            (CMD_START_SERVER2, Some(_submatches)) => {
                Cli::start_server2(driver, options).map(|_| 0)
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

fn configure() -> DriverResult<(Box<dyn Driver>, CliOptions)> {
    let database_url = env::string(ENV_DATABASE_URL)?;
    let database_connections = env::value_opt::<u32>(ENV_DATABASE_CONNECTIONS)?;
    let server_threads = env::value_opt::<usize>(ENV_SERVER_THREADS)?.unwrap_or(4);
    let server_hostname = env::string_opt(ENV_SERVER_HOSTNAME).unwrap_or_else(|| "sso".to_owned());
    let server_bind = env::string(ENV_SERVER_BIND)?;
    let smtp = env::smtp(
        ENV_SMTP_HOST,
        ENV_SMTP_PORT,
        ENV_SMTP_USER,
        ENV_SMTP_PASSWORD,
    )?;
    let smtp_file = env::string_opt(ENV_SMTP_FILE);
    let password_pwned_enabled =
        env::value_opt::<bool>(ENV_PASSWORD_PWNED_ENABLED)?.unwrap_or(false);
    let github =
        ServerOptionsProvider::new(env::oauth2(ENV_GITHUB_CLIENT_ID, ENV_GITHUB_CLIENT_SECRET)?);
    let microsoft = ServerOptionsProvider::new(env::oauth2(
        ENV_MICROSOFT_CLIENT_ID,
        ENV_MICROSOFT_CLIENT_SECRET,
    )?);
    let rustls = env::rustls(
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

    let server = ServerOptions::new(server_bind)
        .set_hostname(&server_hostname)
        .set_password_pwned_enabled(password_pwned_enabled)
        .set_provider(ServerOptionsProviderGroup::new(github, microsoft))
        .set_rustls(rustls)
        .set_user_agent(&server_hostname)
        .set_smtp_transport(smtp)
        .set_smtp_file_transport(smtp_file);
    let options = CliOptions::new(server_threads, server);

    Ok((driver, options))
}
