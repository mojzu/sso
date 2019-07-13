use crate::core;
use crate::driver::Driver;

pub fn secret_key_create(_driver: Box<Driver>, _secret_key: &str) -> Result<(), core::Error> {
    Ok(())
}

pub fn secret_key_verify(_driver: Box<Driver>, _secret_key: &str) -> Result<(), core::Error> {
    Ok(())
}

pub fn disk_status(_driver: Box<Driver>, _disk: Option<&str>) -> Result<(), core::Error> {
    Ok(())
}

pub fn disk_create(
    _driver: Box<Driver>,
    _disk: &str,
    _secret_key: &str,
    _version_retention: Option<&str>,
    _duration_retention: Option<&str>,
) -> Result<(), core::Error> {
    Ok(())
}

pub fn disk_list(_driver: Box<Driver>) -> Result<(), core::Error> {
    Ok(())
}

pub fn key_status(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), core::Error> {
    Ok(())
}

pub fn key_list(_driver: Box<Driver>, _disk: &str) -> Result<(), core::Error> {
    Ok(())
}

pub fn version_list(_driver: Box<Driver>, _disk: &str, _key: &str) -> Result<(), core::Error> {
    Ok(())
}
