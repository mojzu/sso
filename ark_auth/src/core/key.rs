use crate::core::{Error, Key, Service};
use crate::driver;

/// List keys where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<Key>, Error> {
    unimplemented!();
}

/// List keys where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<Key>, Error> {
    unimplemented!();
}

/// Create key.
pub fn create(
    driver: &driver::Driver,
    service: &Service,
    name: &str,
    user_id: Option<i64>,
) -> Result<Key, Error> {
    unimplemented!();
}

/// Read key by ID.
/// TODO(refactor): Use Option here?
pub fn read_by_id(driver: &driver::Driver, service: &Service, id: i64) -> Result<Key, Error> {
    unimplemented!();
}

/// Update key by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<Key, Error> {
    unimplemented!();
}

/// Delete key by ID.
pub fn delete_by_id(driver: &driver::Driver, service: &Service, id: i64) -> Result<usize, Error> {
    unimplemented!();
}
