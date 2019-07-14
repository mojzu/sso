#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;

use ark_kv::{cli, driver, driver::Driver};
use clap::{App, Arg, SubCommand};

const COMMAND_SECRET_KEY: &str = "secret-key";
const COMMAND_DISK: &str = "disk";
const COMMAND_KEY: &str = "key";
const COMMAND_STATUS: &str = "status";
const COMMAND_CREATE: &str = "create";
const COMMAND_LIST: &str = "list";
const COMMAND_READ: &str = "read";
const COMMAND_WRITE: &str = "write";
const COMMAND_VERIFY: &str = "verify";
const COMMAND_DELETE: &str = "delete";
const COMMAND_POLL: &str = "poll";
const COMMAND_MOUNT: &str = "mount";

const ARG_SECRET_KEY: &str = "SECRET_KEY";
const ARG_DISK: &str = "DISK";
const ARG_KEY: &str = "KEY";
const ARG_VERSION_RETENTION: &str = "VERSION_RETENTION";
const ARG_DURATION_RETENTION: &str = "DURATION_RETENTION";
const ARG_FILE: &str = "FILE";
const ARG_DIRECTORY: &str = "DIRECTORY";
const ARG_STR: &str = "STR";
const ARG_VACUUM: &str = "VACUUM";
const ARG_MOUNTPOINT: &str = "MOUNTPOINT";

const HELP_SECRET_KEY: &str = "Secret key file";
const HELP_DISK: &str = "Disk name";
const HELP_KEY: &str = "Key name";
const HELP_MOUNTPOINT: &str = "Path to mount disk";

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
                .about("Create")
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
                .about("Verify")
                .author(crate_authors!("\n"))
                .subcommands(vec![
                    SubCommand::with_name(COMMAND_SECRET_KEY)
                        .version(crate_version!())
                        .about("Verify a secret key")
                        .author(crate_authors!("\n"))
                        .arg(
                            Arg::with_name(ARG_SECRET_KEY)
                                .help(HELP_SECRET_KEY)
                                .required(true)
                                .index(1),
                        ),
                    SubCommand::with_name(COMMAND_KEY)
                        .version(crate_version!())
                        .about("Verify value of key in disk")
                        .author(crate_authors!("\n"))
                        .args(&[
                            Arg::with_name(ARG_DISK)
                                .help(HELP_DISK)
                                .required(true)
                                .index(1),
                            Arg::with_name(ARG_KEY)
                                .help(HELP_KEY)
                                .required(true)
                                .index(2),
                            Arg::with_name(ARG_SECRET_KEY)
                                .help(HELP_SECRET_KEY)
                                .required(true)
                                .index(3),
                            clap::Arg::with_name(ARG_STR)
                                .long("str")
                                .help("Compare to string")
                                .takes_value(true)
                                .required(false),
                            clap::Arg::with_name(ARG_FILE)
                                .long("file")
                                .help("Compare to file")
                                .takes_value(true)
                                .required(false),
                        ]),
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
            SubCommand::with_name(COMMAND_READ)
                .version(crate_version!())
                .about("Read")
                .author(crate_authors!("\n"))
                .subcommands(vec![
                    SubCommand::with_name(COMMAND_DISK)
                        .version(crate_version!())
                        .about("Read values of keys in disk")
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
                            clap::Arg::with_name(ARG_DIRECTORY)
                                .long("directory")
                                .help("Write to directory")
                                .takes_value(true)
                                .required(false),
                        ]),
                    SubCommand::with_name(COMMAND_KEY)
                        .version(crate_version!())
                        .about("Read value of key in disk")
                        .author(crate_authors!("\n"))
                        .args(&[
                            Arg::with_name(ARG_DISK)
                                .help(HELP_DISK)
                                .required(true)
                                .index(1),
                            Arg::with_name(ARG_KEY)
                                .help(HELP_KEY)
                                .required(true)
                                .index(2),
                            Arg::with_name(ARG_SECRET_KEY)
                                .help(HELP_SECRET_KEY)
                                .required(true)
                                .index(3),
                            clap::Arg::with_name(ARG_FILE)
                                .long("file")
                                .help("Write to file")
                                .takes_value(true)
                                .required(false),
                        ]),
                ]),
            SubCommand::with_name(COMMAND_WRITE)
                .version(crate_version!())
                .about("Write")
                .author(crate_authors!("\n"))
                .subcommands(vec![
                    SubCommand::with_name(COMMAND_DISK)
                        .version(crate_version!())
                        .about("Write values to keys in disk")
                        .author(crate_authors!("\n"))
                        .args(&[
                            Arg::with_name(ARG_DISK)
                                .help(HELP_DISK)
                                .required(true)
                                .index(1),
                            clap::Arg::with_name(ARG_DIRECTORY)
                                .long("directory")
                                .help("Write from directory")
                                .takes_value(true)
                                .required(false),
                        ]),
                    SubCommand::with_name(COMMAND_KEY)
                        .version(crate_version!())
                        .about("Write value to key in disk")
                        .author(crate_authors!("\n"))
                        .args(&[
                            Arg::with_name(ARG_DISK)
                                .help(HELP_DISK)
                                .required(true)
                                .index(1),
                            Arg::with_name(ARG_KEY)
                                .help(HELP_KEY)
                                .required(true)
                                .index(2),
                            clap::Arg::with_name(ARG_STR)
                                .long("str")
                                .help("Write from string")
                                .takes_value(true)
                                .required(false),
                            clap::Arg::with_name(ARG_FILE)
                                .long("file")
                                .help("Write from file")
                                .takes_value(true)
                                .required(false),
                        ]),
                ]),
            SubCommand::with_name(COMMAND_DELETE)
                .version(crate_version!())
                .about("Delete disk, or key in disk")
                .author(crate_authors!("\n"))
                .args(&[
                    Arg::with_name(ARG_DISK)
                        .help(HELP_DISK)
                        .required(true)
                        .index(1),
                    Arg::with_name(ARG_KEY)
                        .help(HELP_KEY)
                        .required(false)
                        .index(2),
                ]),
            SubCommand::with_name(COMMAND_POLL)
                .version(crate_version!())
                .about("Poll disks")
                .author(crate_authors!("\n"))
                .arg(
                    Arg::with_name(ARG_VACUUM)
                        .long("vacuum")
                        .help("Vacuum database")
                        .required(false),
                ),
            SubCommand::with_name(COMMAND_MOUNT)
                .version(crate_version!())
                .about("Mount disk as FUSE filesystem")
                .author(crate_authors!("\n"))
                .args(&[
                    clap::Arg::with_name(ARG_DISK)
                        .help(HELP_DISK)
                        .required(true)
                        .index(1),
                    clap::Arg::with_name(ARG_MOUNTPOINT)
                        .help(HELP_MOUNTPOINT)
                        .required(true)
                        .index(2),
                ]),
        ])
        .get_matches();

    // Build configuration and driver from environment.
    let (_configuration, driver) = configuration_from_environment();

    // Call library functions with command line arguments.
    let result = match matches.subcommand() {
        (COMMAND_CREATE, Some(submatches)) => match submatches.subcommand() {
            (COMMAND_SECRET_KEY, Some(submatches)) => {
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                cli::secret_key_create(driver, secret_key).map(|result| {
                    println!("create-secret-key {:?}", result);
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
                .map(|result| {
                    println!("create-disk {:?}", result);
                    0
                })
            }
            _ => {
                println!("{}", submatches.usage());
                Ok(1)
            }
        },
        (COMMAND_VERIFY, Some(submatches)) => match submatches.subcommand() {
            (COMMAND_SECRET_KEY, Some(submatches)) => {
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                cli::secret_key_verify(driver, secret_key).map(|result| {
                    println!("verify-secret-key {:?}", result);
                    0
                })
            }
            (COMMAND_KEY, Some(submatches)) => {
                let disk = submatches.value_of(ARG_DISK).unwrap();
                let key = submatches.value_of(ARG_KEY).unwrap();
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                match submatches.value_of(ARG_STR) {
                    Some(s) => cli::key_verify_from_string(driver, disk, key, secret_key, s),
                    None => match submatches.value_of(ARG_FILE) {
                        Some(f) => cli::key_verify_from_file(driver, disk, key, secret_key, f),
                        None => cli::key_verify_from_stdin(driver, disk, key, secret_key),
                    },
                }
                .map(|result| {
                    println!("verify-key {:?}", result);
                    0
                })
            }
            _ => {
                println!("{}", submatches.usage());
                Ok(1)
            }
        },
        (COMMAND_LIST, Some(submatches)) => match submatches.value_of(ARG_DISK) {
            Some(disk) => match submatches.value_of(ARG_KEY) {
                Some(key) => cli::version_list(driver, disk, key).map(|result| {
                    println!("list-version {:?}", result);
                    0
                }),
                None => cli::key_list(driver, disk).map(|result| {
                    println!("list-key {:?}", result);
                    0
                }),
            },
            None => cli::disk_list(driver).map(|result| {
                println!("list-disk {:?}", result);
                0
            }),
        },
        (COMMAND_STATUS, Some(submatches)) => match submatches.value_of(ARG_DISK) {
            Some(disk) => match submatches.value_of(ARG_KEY) {
                Some(key) => cli::key_status(driver, disk, key).map(|result| {
                    println!("status-key {:?}", result);
                    0
                }),
                None => cli::disk_status(driver, Some(disk)).map(|result| {
                    println!("status-disk {:?}", result);
                    0
                }),
            },
            None => cli::disk_status(driver, None).map(|result| {
                println!("status-disk {:?}", result);
                0
            }),
        },
        (COMMAND_READ, Some(submatches)) => match submatches.subcommand() {
            (COMMAND_DISK, Some(submatches)) => {
                let disk = submatches.value_of(ARG_DISK).unwrap();
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                match submatches.value_of(ARG_DIRECTORY) {
                    Some(d) => cli::disk_read_to_directory(driver, disk, secret_key, d),
                    None => cli::disk_read_to_stdout(driver, disk, secret_key),
                }
                .map(|result| {
                    println!("read-disk {:?}", result);
                    0
                })
            }
            (COMMAND_KEY, Some(submatches)) => {
                let disk = submatches.value_of(ARG_DISK).unwrap();
                let key = submatches.value_of(ARG_KEY).unwrap();
                let secret_key = submatches.value_of(ARG_SECRET_KEY).unwrap();
                match submatches.value_of(ARG_FILE) {
                    Some(f) => cli::key_read_to_file(driver, disk, key, secret_key, f),
                    None => cli::key_read_to_stdout(driver, disk, key, secret_key),
                }
                .map(|result| {
                    println!("read-key {:?}", result);
                    0
                })
            }
            _ => {
                println!("{}", submatches.usage());
                Ok(1)
            }
        },
        (COMMAND_WRITE, Some(submatches)) => match submatches.subcommand() {
            (COMMAND_DISK, Some(submatches)) => {
                let disk = submatches.value_of(ARG_DISK).unwrap();
                match submatches.value_of(ARG_DIRECTORY) {
                    Some(d) => cli::disk_write_from_directory(driver, disk, d),
                    None => cli::disk_write_from_stdin(driver, disk),
                }
                .map(|result| {
                    println!("write-disk {:?}", result);
                    0
                })
            }
            (COMMAND_KEY, Some(submatches)) => {
                let disk = submatches.value_of(ARG_DISK).unwrap();
                let key = submatches.value_of(ARG_KEY).unwrap();
                match submatches.value_of(ARG_STR) {
                    Some(s) => cli::key_write_from_string(driver, disk, key, s),
                    None => match submatches.value_of(ARG_FILE) {
                        Some(f) => cli::key_write_from_file(driver, disk, key, f),
                        None => cli::key_write_from_stdin(driver, disk, key),
                    },
                }
                .map(|result| {
                    println!("write-key {:?}", result);
                    0
                })
            }
            _ => {
                println!("{}", submatches.usage());
                Ok(1)
            }
        },
        (COMMAND_DELETE, Some(submatches)) => {
            let disk = submatches.value_of(ARG_DISK).unwrap();
            match submatches.value_of(ARG_KEY) {
                Some(key) => cli::key_delete(driver, disk, key).map(|result| {
                    println!("delete-key {:?}", result);
                    0
                }),
                None => cli::disk_delete(driver, disk).map(|result| {
                    println!("delete-disk {:?}", result);
                    0
                }),
            }
        }
        (COMMAND_POLL, Some(submatches)) => {
            let vacuum = submatches.is_present(ARG_VACUUM);
            cli::poll(driver, vacuum).map(|result| {
                println!("vacuum {:?}", result);
                0
            })
        }
        (COMMAND_MOUNT, Some(submatches)) => {
            let disk = submatches.value_of(ARG_DISK).unwrap();
            let mountpoint = submatches.value_of(ARG_MOUNTPOINT).unwrap();
            cli::mount(driver, disk, mountpoint).map(|result| {
                println!("mount {:?}", result);
                0
            })
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
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "db.sqlite3".to_owned());
    let database_connections =
        std::env::var("DATABASE_CONNECTIONS").unwrap_or_else(|_| "1".to_owned());
    let database_connections = database_connections.parse::<u32>().unwrap();

    let configuration = Configuration { database_url };
    let driver =
        driver::SqliteDriver::initialise(&configuration.database_url, database_connections)
            .unwrap()
            .box_clone();
    (configuration, driver)
}
