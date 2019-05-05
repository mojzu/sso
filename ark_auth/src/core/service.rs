use crate::driver;
use crate::{
    core,
    core::{Error, Service},
};

/// Authenticate service key and return associated service.
pub fn authenticate(driver: &driver::Driver, key_value: Option<String>) -> Result<Service, Error> {
    match key_value {
        Some(key_value) => core::key::read_by_service_value(driver, &key_value)
            .and_then(|key| key.ok_or_else(|| Error::Forbidden))
            .and_then(|key| {
                driver
                    .service_read_by_id(key.service_id)
                    .map_err(Error::Driver)
            })
            .and_then(|service| service.ok_or_else(|| Error::Forbidden)),
        None => Err(Error::Forbidden),
    }
}

/// Create service.
pub fn create(driver: &driver::Driver, name: &str, url: &str) -> Result<Service, Error> {
    driver.service_create(name, url).map_err(Error::Driver)
}

/// Read service by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service: &Service,
    id: i64,
) -> Result<Option<Service>, Error> {
    driver.service_read_by_id(id).map_err(Error::Driver)
}

/// Update service by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<Service, Error> {
    driver.service_update_by_id(id, name).map_err(Error::Driver)
}

/// Delete service by ID.
pub fn delete_by_id(driver: &driver::Driver, _service: &Service, id: i64) -> Result<usize, Error> {
    driver.service_delete_by_id(id).map_err(Error::Driver)
}
