use crate::core;
use crate::driver::Driver;
use std::path::Path;

/// Command line interface errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Core error wrapper.
    #[fail(display = "CliError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
}

pub fn secret_key_create(_driver: Box<Driver>, secret_key: &str) -> Result<String, Error> {
    let path = Path::new(secret_key);
    core::DiskEncryption::new(1)
        .write_to_file(path)
        .map_err(Error::Core)
}

pub fn secret_key_verify(_driver: Box<Driver>, secret_key: &str) -> Result<bool, Error> {
    let path = Path::new(secret_key);
    core::DiskEncryption::read_from_file(path)
        .map_err(Error::Core)?
        .verify()
        .map_err(Error::Core)
}

pub fn disk_list(_driver: Box<Driver>) -> Result<(), Error> {
    Ok(())
}

pub fn disk_status(_driver: Box<Driver>, _disk: Option<&str>) -> Result<(), Error> {
    Ok(())
}

pub fn disk_create(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
    _version_retention: Option<&str>,
    _duration_retention: Option<&str>,
) -> Result<(), Error> {
    Ok(())
}

pub fn disk_read_to_directory(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
    _d: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn disk_read_to_stdout(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn disk_write_from_directory(_driver: Box<Driver>, _disk: &str, _d: &str) -> Result<(), Error> {
    Ok(())
}

pub fn disk_write_from_stdin(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    Ok(())
}

pub fn disk_delete(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    Ok(())
}

pub fn key_list(_driver: Box<Driver>, _disk: &str) -> Result<(), Error> {
    Ok(())
}

pub fn key_status(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    Ok(())
}

pub fn key_read_to_file(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
    _f: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_read_to_stdout(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_write_from_string(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _s: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_write_from_file(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _f: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_write_from_stdin(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    Ok(())
}

pub fn key_verify_from_string(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
    _s: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_verify_from_file(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
    _f: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_verify_from_stdin(
    _driver: Box<Driver>,
    _disk: &str,
    _key: &str,
    _secret_key: &str,
) -> Result<(), Error> {
    Ok(())
}

pub fn key_delete(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    Ok(())
}

pub fn version_list(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), Error> {
    Ok(())
}

pub fn poll(_driver: Box<Driver>, _vacuum: bool) -> Result<(), Error> {
    Ok(())
}

pub fn mount(_driver: Box<Driver>, _disk: &str, _mountpoint: &str) -> Result<(), Error> {
    Ok(())
}
