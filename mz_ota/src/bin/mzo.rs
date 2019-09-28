//! Binary application.
#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use mz_ota::{cli, core};
use sentry::integrations::log::LoggerOptions;

const CRATE_NAME: &str = crate_name!();
const CRATE_VERSION: &str = crate_version!();
const CRATE_DESCRIPTION: &str = crate_description!();
const CRATE_AUTHORS: &str = "Sam Ward <git@mojzu.net>";

const COMMAND_CREATE_KEY: &str = "create-key";
const ARG_NAME: &str = "NAME";

/// Main errors.
#[derive(Debug, Fail)]
enum Error {
    /// Command invalid.
    #[fail(display = "MainError::CommandInvalid")]
    CommandInvalid,
    /// Core error wrapper.
    #[fail(display = "MainError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
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
    let matches = App::new(CRATE_NAME)
        .version(CRATE_VERSION)
        .about(CRATE_DESCRIPTION)
        .author(CRATE_AUTHORS)
        .subcommands(vec![SubCommand::with_name(COMMAND_CREATE_KEY)
            .version(CRATE_VERSION)
            .about("Create key")
            .author(CRATE_AUTHORS)
            .arg(
                Arg::with_name(ARG_NAME)
                    .help("Key name")
                    .required(true)
                    .index(1),
            )])
        .get_matches();

    // Initialise core.
    // TODO(refactor): Configuration from environment.
    core::init().unwrap();

    // Call library functions with command line arguments.
    let result = match matches.subcommand() {
        (COMMAND_CREATE_KEY, Some(submatches)) => {
            let name = submatches.value_of(ARG_NAME).unwrap();
            cli::create_key(name).map_err(Error::Core).map(|path| {
                println!("path: {}", path);
                0
            })
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
