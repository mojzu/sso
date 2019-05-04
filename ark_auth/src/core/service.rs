use crate::core::{Error, Service};
use crate::driver;

/// Authenticate service key and return associated service.
pub fn authenticate(driver: &driver::Driver, key_value: Option<String>) -> Result<Service, Error> {
    match key_value {
        Some(key_value) => driver
            .service_read_by_key_value(&key_value)
            .map_err(Error::Driver)
            .and_then(|service| service.ok_or_else(|| Error::Forbidden)),
        None => Err(Error::Forbidden),
    }
}

/// List services where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<Service>, Error> {
    unimplemented!();
}

/// List services where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<Service>, Error> {
    unimplemented!();
}

/// Create service.
pub fn create(driver: &driver::Driver, name: &str, url: &str) -> Result<Service, Error> {
    unimplemented!();
}

/// Read service by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
) -> Result<Option<Service>, Error> {
    unimplemented!();
}

/// Update service by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<Service, Error> {
    unimplemented!();
}

/// Delete service by ID.
pub fn delete_by_id(driver: &driver::Driver, service: &Service, id: i64) -> Result<usize, Error> {
    unimplemented!();
}
