use crate::core;
use crate::driver::Driver;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Command line interface errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Core error wrapper.
    #[fail(display = "CliError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Standard IO error wrapper.
    #[fail(display = "CliError::StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
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

pub fn disk_list(driver: Box<Driver>) -> Result<Vec<core::Disk>, Error> {
    core::disk::list(driver.as_ref()).map_err(Error::Core)
}

pub fn disk_status(_driver: Box<Driver>, _disk: Option<&str>) -> Result<(), Error> {
    unimplemented!();
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
        .chunk_size(536870912)
        .compression("zlib".to_owned())
        .encryption(disk_encryption.new_internal())
        .version_retention(version_retention)
        .duration_retention(duration_retention)
        .build()
        .unwrap();

    core::disk::create(driver.as_ref(), disk, &options).map_err(Error::Core)
}

pub fn disk_read_to_directory(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
    _d: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn disk_read_to_stdout(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn disk_write_from_directory(_driver: Box<Driver>, _disk: &str, _d: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn disk_write_from_stdin(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn disk_delete(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_list(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_status(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_read_to_file(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
    _f: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_read_to_stdout(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    unimplemented!();
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
    // TODO(refactor): Support modified time here.
    let file = File::open(f).map_err(Error::StdIo)?;
    let mut reader = BufReader::new(file);
    let options = core::KeyWriteOptionsBuilder::default().build().unwrap();
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
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
    _s: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_verify_from_file(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
    _f: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_verify_from_stdin(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    unimplemented!();
}

pub fn key_delete(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn version_list(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    unimplemented!();
}

pub fn poll(_driver: Box<Driver>, _vacuum: bool) -> Result<(), Error> {
    unimplemented!();
}

pub fn mount(_driver: Box<Driver>, _disk: &str, _mountpoint: &str) -> Result<(), Error> {
    unimplemented!();
}
