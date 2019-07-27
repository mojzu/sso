use crate::core::{Error, Key, Version};
use crate::driver::Driver;

pub fn create(driver: &Driver, key: &Key, hash: &[u8], size: i64) -> Result<Version, Error> {
    driver
        .version_create(hash, size, &key.id)
        .map_err(Error::Driver)
}

pub fn read_by_key(driver: &Driver, key: &Key) -> Result<Version, Error> {
    read_opt_by_key(driver, key).and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn read_opt_by_key(driver: &Driver, key: &Key) -> Result<Option<Version>, Error> {
    match &key.version_id {
        Some(version_id) => driver.version_read_by_id(version_id).map_err(Error::Driver),
        None => Ok(None),
    }
}
