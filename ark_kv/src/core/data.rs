use crate::core::{Data, Error, Version};
use crate::driver::Driver;

pub fn create(driver: &Driver, version: &Version, chunk: i64, data: &[u8]) -> Result<Data, Error> {
    unimplemented!();
}
