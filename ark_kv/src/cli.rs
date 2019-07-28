use crate::core;
use crate::driver::Driver;
use glob::glob;
use std::convert::TryInto;
use std::fs::{symlink_metadata, File};
use std::io::BufReader;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Command line interface errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Not a file error.
    #[fail(display = "CliError::IsNotFile {}", _0)]
    IsNotFile(String),
    /// Core error wrapper.
    #[fail(display = "CliError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Standard IO error wrapper.
    #[fail(display = "CliError::StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
    /// Standard environment variable error wrapper.
    #[fail(display = "CliError::StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),
    /// Standard number parse integer error wrapper.
    #[fail(display = "CliError::StdNumParseInt {}", _0)]
    StdNumParseInt(#[fail(cause)] std::num::ParseIntError),
    /// Glob error wrapper.
    #[fail(display = "CliError::Glob {}", _0)]
    Glob(#[fail(cause)] glob::GlobError),
}

/// Read optional environment variable string value.
pub fn opt_str_from_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Read optional environment variable u32 value.
/// Logs an error message in case value is not a valid unsigned integer.
pub fn opt_u32_from_env(name: &str) -> Result<Option<u32>, Error> {
    let value = std::env::var(name).ok();
    if let Some(x) = value {
        match x.parse::<u32>() {
            Ok(x) => Ok(Some(x)),
            Err(err) => {
                error!("{} is invalid unsigned integer ({})", name, err);
                Err(Error::StdNumParseInt(err))
            }
        }
    } else {
        Ok(None)
    }
}

pub fn secret_key_create(_driver: Box<Driver>, secret_key: &str) -> Result<String, Error> {
    let secret_key_path = Path::new(secret_key);
    core::DiskEncryption::new(1)
        .write_to_file(secret_key_path)
        .map_err(Error::Core)
}

pub fn secret_key_verify(_driver: Box<Driver>, secret_key: &str) -> Result<bool, Error> {
    let secret_key_path = Path::new(secret_key);
    core::DiskEncryption::read_from_file(secret_key_path)
        .map_err(Error::Core)?
        .verify()
        .map_err(Error::Core)
}

pub fn status(driver: Box<Driver>) -> Result<core::Status, Error> {
    core::status(driver.as_ref()).map_err(Error::Core)
}

pub fn disk_list(driver: Box<Driver>) -> Result<Vec<core::Disk>, Error> {
    core::disk::list(driver.as_ref()).map_err(Error::Core)
}

pub fn disk_status(driver: Box<Driver>, disk: &str) -> Result<core::DiskStatus, Error> {
    core::disk::status(driver.as_ref(), disk).map_err(Error::Core)
}

pub fn disk_create(
    driver: Box<Driver>,
    disk: &str,
    secret_key: &str,
    version_retention: Option<&str>,
    duration_retention: Option<&str>,
) -> Result<core::Disk, Error> {
    let secret_key_path = Path::new(secret_key);
    let disk_encryption =
        core::DiskEncryption::read_from_file(secret_key_path).map_err(Error::Core)?;
    let version_retention = match version_retention {
        Some(version_retention) => version_retention.parse::<i64>().unwrap(),
        None => 0,
    };
    let duration_retention = match duration_retention {
        Some(duration_retention) => duration_retention.parse::<i64>().unwrap(),
        None => 0,
    };
    let options = core::DiskOptionsBuilder::default()
        .chunk_size(536_870_912)
        .compression("zlib".to_owned())
        .encryption(disk_encryption.new_internal())
        .version_retention(version_retention)
        .duration_retention(duration_retention)
        .build()
        .unwrap();

    core::disk::create(driver.as_ref(), disk, &options).map_err(Error::Core)
}

pub fn disk_read_to_directory(
    driver: Box<Driver>,
    disk: &str,
    secret_key: &str,
    d: &str,
) -> Result<(), Error> {
    let secret_key = Path::new(secret_key);
    let disk_encryption = core::DiskEncryption::read_from_file(secret_key).map_err(Error::Core)?;
    let disk = core::disk::read_by_name(driver.as_ref(), disk).map_err(Error::Core)?;
    let keys = core::key::list(driver.as_ref(), &disk).map_err(Error::Core)?;

    for key in keys.into_iter() {
        let file_path = Path::new(d).join(&key.name);
        let parent_path = file_path.parent().unwrap();
        std::fs::create_dir_all(parent_path).unwrap();

        let mut file = File::create(&file_path).map_err(Error::StdIo)?;
        core::key::read(
            driver.as_ref(),
            &disk.name,
            &key.name,
            &disk_encryption,
            &mut file,
        )
        .map_err(Error::Core)?;
    }

    Ok(())
}

pub fn disk_read_to_stdout(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn disk_write_from_directory(driver: Box<Driver>, disk: &str, d: &str) -> Result<(), Error> {
    let directory_prefix = Path::new(d).canonicalize().unwrap();
    let glob_path = Path::new(d).join("**/*");
    let pattern = glob_path.to_str().unwrap();

    for entry in glob(pattern).unwrap() {
        match entry {
            Ok(path) => {
                let options = file_key_write_options(path.as_path())?;

                if let Some(options) = options {
                    let path = path.canonicalize().unwrap();
                    let key = path.strip_prefix(&directory_prefix).unwrap();
                    let key_name = key.to_str().unwrap();

                    let file = File::open(&path).map_err(Error::StdIo)?;
                    let mut reader = BufReader::new(file);
                    core::key::write(driver.as_ref(), disk, key_name, &mut reader, options)
                        .map_err(Error::Core)?;
                }
            }
            Err(err) => {
                return Err(Error::Glob(err));
            }
        }
    }

    Ok(())
}

pub fn disk_write_from_stdin(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn disk_delete(driver: Box<Driver>, disk: &str) -> Result<usize, Error> {
    core::disk::delete(driver.as_ref(), disk).map_err(Error::Core)
}

pub fn key_list(driver: Box<Driver>, disk: &str) -> Result<Vec<core::Key>, Error> {
    let disk = core::disk::read_by_name(driver.as_ref(), disk).map_err(Error::Core)?;
    core::key::list(driver.as_ref(), &disk).map_err(Error::Core)
}

pub fn key_status(driver: Box<Driver>, disk: &str, key: &str) -> Result<core::KeyStatus, Error> {
    let disk = core::disk::read_by_name(driver.as_ref(), disk).map_err(Error::Core)?;
    core::key::status(driver.as_ref(), &disk, key).map_err(Error::Core)
}

pub fn key_read_to_file(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    secret_key: &str,
    f: &str,
) -> Result<(core::Key, core::Version), Error> {
    let secret_key = Path::new(secret_key);
    let disk_encryption = core::DiskEncryption::read_from_file(secret_key).map_err(Error::Core)?;
    let file_path = Path::new(f);
    let mut file = File::create(&file_path).map_err(Error::StdIo)?;
    core::key::read(driver.as_ref(), disk, key, &disk_encryption, &mut file).map_err(Error::Core)
}

pub fn key_read_to_stdout(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    secret_key: &str,
) -> Result<(core::Key, core::Version), Error> {
    let secret_key = Path::new(secret_key);
    let disk_encryption = core::DiskEncryption::read_from_file(secret_key).map_err(Error::Core)?;
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    core::key::read(driver.as_ref(), disk, key, &disk_encryption, &mut writer).map_err(Error::Core)
}

pub fn key_write_from_string(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    s: &str,
) -> Result<(core::Key, core::Version), Error> {
    let mut reader = BufReader::new(s.as_bytes());
    let options = core::KeyWriteOptionsBuilder::default().build().unwrap();
    core::key::write(driver.as_ref(), disk, key, &mut reader, options).map_err(Error::Core)
}

pub fn key_write_from_file(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    f: &str,
) -> Result<(core::Key, core::Version), Error> {
    let path = Path::new(f);
    let options = file_key_write_options(&path)?.unwrap();
    let file = File::open(f).map_err(Error::StdIo)?;
    let mut reader = BufReader::new(file);
    core::key::write(driver.as_ref(), disk, key, &mut reader, options).map_err(Error::Core)
}

pub fn key_write_from_stdin(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
) -> Result<(core::Key, core::Version), Error> {
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let options = core::KeyWriteOptionsBuilder::default().build().unwrap();
    core::key::write(driver.as_ref(), disk, key, &mut reader, options).map_err(Error::Core)
}

pub fn key_verify_from_string(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    secret_key: &str,
    s: &str,
) -> Result<(), Error> {
    let secret_key = Path::new(secret_key);
    let disk_encryption = core::DiskEncryption::read_from_file(secret_key).map_err(Error::Core)?;
    let mut reader = BufReader::new(s.as_bytes());
    core::key::verify(driver.as_ref(), disk, key, &disk_encryption, &mut reader)
        .map_err(Error::Core)
}

pub fn key_verify_from_file(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    secret_key: &str,
    f: &str,
) -> Result<(), Error> {
    let secret_key = Path::new(secret_key);
    let disk_encryption = core::DiskEncryption::read_from_file(secret_key).map_err(Error::Core)?;
    let file = File::open(f).map_err(Error::StdIo)?;
    let mut reader = BufReader::new(file);
    core::key::verify(driver.as_ref(), disk, key, &disk_encryption, &mut reader)
        .map_err(Error::Core)
}

pub fn key_verify_from_stdin(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
    secret_key: &str,
) -> Result<(), Error> {
    let secret_key = Path::new(secret_key);
    let disk_encryption = core::DiskEncryption::read_from_file(secret_key).map_err(Error::Core)?;
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    core::key::verify(driver.as_ref(), disk, key, &disk_encryption, &mut reader)
        .map_err(Error::Core)
}

pub fn key_delete(driver: Box<Driver>, disk: &str, key: &str) -> Result<usize, Error> {
    let disk = core::disk::read_by_name(driver.as_ref(), disk).map_err(Error::Core)?;
    core::key::delete(driver.as_ref(), &disk, key).map_err(Error::Core)
}

pub fn version_list(
    driver: Box<Driver>,
    disk: &str,
    key: &str,
) -> Result<Vec<core::Version>, Error> {
    let disk = core::disk::read_by_name(driver.as_ref(), disk).map_err(Error::Core)?;
    let key = core::key::read_by_name(driver.as_ref(), &disk, key).map_err(Error::Core)?;
    core::version::list(driver.as_ref(), &key).map_err(Error::Core)
}

pub fn poll(driver: Box<Driver>, vacuum: bool) -> Result<(), Error> {
    core::poll(driver.as_ref(), vacuum).map_err(Error::Core)
}

pub fn mount(driver: Box<Driver>, disk: &str, mountpoint: &str) -> Result<(), Error> {
    core::fuse::mount(driver.as_ref(), disk, mountpoint).map_err(Error::Core)
}

fn file_key_write_options(f: &Path) -> Result<Option<core::KeyWriteOptions>, Error> {
    let mut options = core::KeyWriteOptionsBuilder::default();
    let metadata = symlink_metadata(f).map_err(Error::StdIo)?;

    if metadata.is_file() {
        if let Ok(time) = metadata.modified() {
            let modified_time = time.duration_since(UNIX_EPOCH).unwrap();
            let modified_time: i64 = modified_time.as_secs().try_into().unwrap();
            options.check_modified_time(modified_time);
        }

        Ok(Some(options.build().unwrap()))
    } else {
        Ok(None)
    }
}
