use crate::core::{Data, Error, Version};
use crate::driver::Driver;

pub fn create(driver: &Driver, version: &Version, chunk: i64, data: &[u8]) -> Result<Data, Error> {
    driver
        .data_create(chunk, data, &version.id)
        .map_err(Error::Driver)
}

pub fn read_opt_by_version(
    driver: &Driver,
    version: &Version,
    chunk: i64,
) -> Result<Option<Data>, Error> {
    driver
        .data_read_by_chunk(chunk, &version.id)
        .map_err(Error::Driver)
}
