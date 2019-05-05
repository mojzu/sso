use crate::core::{Error, Key, Service, User};
use crate::driver;

// TODO(refactor): Use service for permissions, masking users, keys, etc.

/// List keys where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<Key>, Error> {
    driver
        .key_list_where_id_lt(service.id, lt, limit)
        .map_err(Error::Driver)
}

/// List keys where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<Key>, Error> {
    driver
        .key_list_where_id_gt(service.id, gt, limit)
        .map_err(Error::Driver)
}

/// Create key.
pub fn create(
    driver: &driver::Driver,
    service: &Service,
    name: &str,
    user_id: Option<i64>,
) -> Result<Key, Error> {
    let value = uuid::Uuid::new_v4().to_simple().to_string();
    driver
        .key_create(name, &value, service.id, user_id)
        .map_err(Error::Driver)
}

/// Read key by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service: &Service,
    id: i64,
) -> Result<Option<Key>, Error> {
    driver.key_read_by_id(id).map_err(Error::Driver)
}

/// Read key by user.
pub fn read_by_user(
    driver: &driver::Driver,
    service: &Service,
    user: &User,
) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_user_id(service.id, user.id)
        .map_err(Error::Driver)
}

/// Read key by value (services only).
pub fn read_by_service_value(driver: &driver::Driver, value: &str) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_service_value(value)
        .map_err(Error::Driver)
}

/// Read key by value (users only).
pub fn read_by_user_value(
    driver: &driver::Driver,
    service: &Service,
    value: &str,
) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_user_value(service.id, value)
        .map_err(Error::Driver)
}

/// Update key by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<Key, Error> {
    driver.key_update_by_id(id, name).map_err(Error::Driver)
}

/// Delete key by ID.
pub fn delete_by_id(driver: &driver::Driver, _service: &Service, id: i64) -> Result<usize, Error> {
    driver.key_delete_by_id(id).map_err(Error::Driver)
}
