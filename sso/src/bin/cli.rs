//! Single Sign-On Command Line Interface
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use sentry::integrations::log::LoggerOptions;
use sso::{env, DriverResult, KeyCreate, Postgres, ServiceCreate};

const CRATE_NAME: &str = crate_name!();
const CRATE_VERSION: &str = crate_version!();
const CRATE_DESCRIPTION: &str = crate_description!();
const CRATE_AUTHORS: &str = "Sam Ward <git@mojzu.net>";

/// Sentry URL for logging integration.
const ENV_SENTRY_URL: &str = "SSO_SENTRY_URL";

/// Database connection URL.
const ENV_DATABASE_URL: &str = "SSO_DATABASE_URL";

const CMD_CREATE_ROOT_KEY: &str = "create-root-key";
const CMD_CREATE_SERVICE_WITH_KEY: &str = "create-service-with-key";
const CMD_TASK_RETENTION: &str = "task-retention";

const ARG_NAME: &str = "NAME";
const ARG_URL: &str = "URL";
const ARG_ALLOW_REGISTER: &str = "ALLOW_REGISTER";
const ARG_EMAIL_TEXT: &str = "EMAIL_TEXT";
const ARG_LOCAL_URL: &str = "LOCAL_URL";
const ARG_GITHUB_OAUTH2_URL: &str = "GITHUB_OAUTH2_URL";
const ARG_MICROSOFT_OAUTH2_URL: &str = "MICROSOFT_OAUTH2_URL";
const ARG_WEEKS: &str = "WEEKS";

fn main() {
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
            SubCommand::with_name(CMD_TASK_RETENTION)
                .version(CRATE_VERSION)
                .about("Run retention task")
                .author(CRATE_AUTHORS)
                .arg(
                    Arg::with_name(ARG_WEEKS)
                        .long("weeks")
                        .help("Weeks")
                        .takes_value(true)
                        .required(false),
                ),
        ])
        .get_matches();

    // Build driver from environment variables.
    let result = configure().and_then(|driver| {
        // Call library functions with command line arguments.
        match matches.subcommand() {
            (CMD_CREATE_ROOT_KEY, Some(submatches)) => {
                let name = submatches.value_of(ARG_NAME).unwrap();
                let create = KeyCreate::root(true, name);
                driver.key_create(&create).map(|key| {
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

                let user_allow_register = user_allow_register
                    .unwrap_or("false")
                    .parse::<bool>()
                    .unwrap();
                let service_create = ServiceCreate {
                    is_enabled: true,
                    name: name.to_owned(),
                    url: url.to_owned(),
                    user_allow_register,
                    user_email_text: user_email_text.unwrap_or("").to_owned(),
                    provider_local_url: provider_local_url.map(|x| x.to_owned()),
                    provider_github_oauth2_url: provider_github_oauth2_url.map(|x| x.to_owned()),
                    provider_microsoft_oauth2_url: provider_microsoft_oauth2_url
                        .map(|x| x.to_owned()),
                };
                let service = driver.service_create(&service_create)?;
                let key_create = KeyCreate::service(true, name, service.id);
                let key = driver.key_create(&key_create)?;
                Ok((service, key)).map(|(service, key)| {
                    println!("{}", service);
                    println!("{}", key);
                    0
                })
            }
            (CMD_TASK_RETENTION, Some(submatches)) => {
                let weeks = submatches.value_of(ARG_WEEKS).unwrap_or("12");
                let weeks: i64 = weeks.parse().unwrap();
                let audit_retention = chrono::Duration::weeks(weeks);
                let created_at = chrono::Utc::now() - audit_retention;
                driver.audit_delete(&created_at).map(|deleted| {
                    println!("{}", deleted);
                    0
                })
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

fn configure() -> DriverResult<Postgres> {
    let database_url = env::string(ENV_DATABASE_URL)?;
    let driver = Postgres::initialise(&database_url, Some(1)).unwrap();
    Ok(driver)
}
