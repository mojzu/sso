use crate::core::{Error, Key, Version};
use crate::driver::Driver;
use chrono::Utc;

pub fn list(driver: &dyn Driver, key: &Key) -> Result<Vec<Version>, Error> {
    let now = Utc::now();
    let id_list = driver
        .version_list_where_created_lte(&now, None, 1024, &key.id)
        .map_err(Error::Driver)?;
    let mut version_list: Vec<Version> = Vec::new();
    for id in id_list.into_iter() {
        let version = read_by_id(driver, &id)?;
        version_list.push(version);
    }
    Ok(version_list)
}

pub fn create(driver: &dyn Driver, key: &Key, hash: &[u8], size: i64) -> Result<Version, Error> {
    driver
        .version_create(hash, size, &key.id)
        .map_err(Error::Driver)
}

pub fn read_by_id(driver: &dyn Driver, id: &str) -> Result<Version, Error> {
    read_opt_by_id(driver, id).and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn read_opt_by_id(driver: &dyn Driver, id: &str) -> Result<Option<Version>, Error> {
    driver.version_read_by_id(id).map_err(Error::Driver)
}

pub fn read_by_key(driver: &dyn Driver, key: &Key) -> Result<Version, Error> {
    read_opt_by_key(driver, key).and_then(|x| x.ok_or_else(|| Error::Unwrap))
}

pub fn read_opt_by_key(driver: &dyn Driver, key: &Key) -> Result<Option<Version>, Error> {
    match &key.version_id {
        Some(version_id) => driver.version_read_by_id(version_id).map_err(Error::Driver),
        None => Ok(None),
    }
}

pub fn delete_by_id(driver: &dyn Driver, id: &str) -> Result<usize, Error> {
    driver.version_delete_by_id(id).map_err(Error::Driver)
}
