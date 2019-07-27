use crate::core::{Error, Key, Version};
use crate::driver::Driver;

pub fn create(driver: &Driver, key: &Key, hash: &[u8], size: i64) -> Result<Version, Error> {
    unimplemented!();
}

pub fn read_by_key(driver: &Driver, key: &Key) -> Result<Option<Version>, Error> {
    unimplemented!();
}
