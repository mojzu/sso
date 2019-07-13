#[macro_use]
extern crate clap;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

use ark_kv::{cli, core, driver, driver::Driver};
use clap::{App, Arg, SubCommand};

const COMMAND_SECRET_KEY: &str = "secret-key";
const COMMAND_DISK: &str = "disk";
const COMMAND_STATUS: &str = "status";
const COMMAND_CREATE: &str = "create";
const COMMAND_LIST: &str = "list";
const COMMAND_VERIFY: &str = "verify";
const ARG_SECRET_KEY: &str = "SECRET_KEY";
const ARG_DISK: &str = "DISK";
const ARG_KEY: &str = "KEY";
const ARG_VERSION_RETENTION: &str = "version_retention";
const ARG_DURATION_RETENTION: &str = "duration_retention";
const HELP_SECRET_KEY: &str = "Secret key file";
const HELP_DISK: &str = "Disk name";
const HELP_KEY: &str = "Key name";

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

struct Configuration {
    database_url: String,
}

fn main() {
    // Configure logging environment variables.
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Command line argument parser.
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!("\n"))
        .subcommands(vec![
            SubCommand::with_name(COMMAND_CREATE)
                .version(crate_version!())
                .about("Create secret key or disk")
                .author(crate_authors!("\n"))
                .subcommands(vec![
                    SubCommand::with_name(COMMAND_SECRET_KEY)
                        .version(crate_version!())
                        .about("Create a secret key")
                        .author(crate_authors!("\n"))
                        .arg(
                            Arg::with_name(ARG_SECRET_KEY)
                                .help(HELP_SECRET_KEY)
                                .required(true)
                                .index(1),
                        ),
                    SubCommand::with_name(COMMAND_DISK)
                        .version(crate_version!())
                        .about("Create a disk")
                        .author(crate_authors!("\n"))
                        .args(&[
                            Arg::with_name(ARG_DISK)
                                .help(HELP_DISK)
                                .required(true)
                                .index(1),
                            Arg::with_name(ARG_SECRET_KEY)
                                .help(HELP_SECRET_KEY)
                                .required(true)
                                .index(2),
                            Arg::with_name(ARG_VERSION_RETENTION)
                                .long("version-retention")
                                .help("Version retention configuration (v >= 0)")
                                .takes_value(true)
                                .required(false),
                            Arg::with_name(ARG_DURATION_RETENTION)
                                .long("duration-retention")
                                .help("Duration retention configuration (s >= 0)")
                                .takes_value(true)
                                .required(false),
                        ]),
                ]),
            SubCommand::with_name(COMMAND_VERIFY)
                .version(crate_version!())
                .about("Verify secret key")
                .author(crate_authors!("\n"))
                .subcommands(vec![SubCommand::with_name(COMMAND_SECRET_KEY)
                    .version(crate_version!())
                    .about("Verify a secret key")
                    .author(crate_authors!("\n"))
                    .arg(
                        Arg::with_name(ARG_SECRET_KEY)
                            .help(HELP_SECRET_KEY)
                            .required(true)
                            .index(1),
                    )]),
            SubCommand::with_name(COMMAND_STATUS)
                .version(crate_version!())
                .about("Get status of many disks, one disk, or one key in disk")
                .author(crate_authors!("\n"))
                .args(&[
                    Arg::with_name(ARG_DISK)
                        .help(HELP_DISK)
                        .required(false)
                        .index(1),
                    Arg::with_name(ARG_KEY)
                        .help(HELP_KEY)
                        .required(false)
                        .index(2),
                ]),
            SubCommand::with_name(COMMAND_LIST)
                .version(crate_version!())
                .about("List many disks, or many keys in disk, or many versions of key in disk")
                .author(crate_authors!("\n"))
                .args(&[
                    Arg::with_name(ARG_DISK)
                        .help(HELP_DISK)
                        .required(false)
                        .index(1),
                    Arg::with_name(ARG_KEY)
                        .help(HELP_KEY)
                        .required(false)
                        .index(2),
                ]),
        ])
        .get_matches();

    // Build configuration and driver from environment.
    let (_configuration, driver) = configuration_from_environment().unwrap();

    // Call library functions with command line arguments.
    let result = match matches.subcommand() {
        (COMMAND_CREATE, Some(submatches)) => match submatches.subcommand() {
            (COMMAND_SECRET_KEY, Some(submatches)) => {
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                cli::secret_key_create(driver, secret_key)
                    .map_err(Error::Core)
                    .map(|secret_key| {
                        println!("secret_key_create {:?}", secret_key);
                        0
                    })
            }
            (COMMAND_DISK, Some(submatches)) => {
                let disk = submatches.value_of(ARG_DISK).unwrap();
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                let version_retention = submatches.value_of(ARG_VERSION_RETENTION);
                let duration_retention = submatches.value_of(ARG_DURATION_RETENTION);
                cli::disk_create(
                    driver,
                    disk,
                    secret_key,
                    version_retention,
                    duration_retention,
                )
                .map_err(Error::Core)
                .map(|disk| {
                    println!("disk_create {:?}", disk);
                    0
                })
            }
            _ => Err(Error::CommandInvalid),
        },
        (COMMAND_VERIFY, Some(submatches)) => match submatches.subcommand() {
            (COMMAND_SECRET_KEY, Some(submatches)) => {
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                cli::secret_key_verify(driver, secret_key)
                    .map_err(Error::Core)
                    .map(|secret_key| {
                        println!("secret_key_verify {:?}", secret_key);
                        0
                    })
            }
            _ => Err(Error::CommandInvalid),
        },
        (COMMAND_STATUS, Some(submatches)) => match submatches.value_of(ARG_DISK) {
            Some(disk) => match submatches.value_of(ARG_KEY) {
                Some(key) => {
                    cli::key_status(driver, disk, key)
                        .map_err(Error::Core)
                        .map(|status| {
                            println!("key_status {:?}", status);
                            0
                        })
                }
                None => cli::disk_status(driver, Some(disk))
                    .map_err(Error::Core)
                    .map(|status| {
                        println!("disk_status {:?}", status);
                        0
                    }),
            },
            None => cli::disk_status(driver, None)
                .map_err(Error::Core)
                .map(|status| {
                    println!("disk_status {:?}", status);
                    0
                }),
        },
        (COMMAND_LIST, Some(submatches)) => match submatches.value_of(ARG_DISK) {
            Some(disk) => match submatches.value_of(ARG_KEY) {
                Some(key) => cli::version_list(driver, disk, key)
                    .map_err(Error::Core)
                    .map(|version_list| {
                        println!("version_list {:?}", version_list);
                        0
                    }),
                None => cli::key_list(driver, disk)
                    .map_err(Error::Core)
                    .map(|key_list| {
                        println!("key_list {:?}", key_list);
                        0
                    }),
            },
            None => cli::disk_list(driver)
                .map_err(Error::Core)
                .map(|disk_list| {
                    println!("disk_list {:?}", disk_list);
                    0
                }),
        },
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
fn configuration_from_environment() -> Result<(Configuration, Box<Driver>), Error> {
    // TODO(refactor): Clean this up, improve error messages.
    let database_url = std::env::var("DATABASE_URL").unwrap();
    let database_connections =
        std::env::var("DATABASE_CONNECTIONS").unwrap_or_else(|_| "1".to_owned());
    let database_connections = database_connections.parse::<u32>().unwrap();

    let configuration = Configuration { database_url };
    let driver =
        driver::SqliteDriver::initialise(&configuration.database_url, database_connections)
            .unwrap()
            .box_clone();
    Ok((configuration, driver))
}
