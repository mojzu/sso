use crate::core::{Error, Key, Service, User};
use crate::driver;

// TODO(refactor): Use service for permissions, masking users, keys, etc. Add tests for this.

/// Authenticate root key.
pub fn authenticate_root(driver: &driver::Driver, key_value: Option<String>) -> Result<(), Error> {
    match key_value {
        Some(key_value) => read_by_root_value(driver, &key_value).and_then(|key| {
            key.ok_or_else(|| Error::Forbidden)?;
            Ok(())
        }),
        None => Err(Error::Forbidden),
    }
}

/// Authenticate service key.
pub fn authenticate_service(
    driver: &driver::Driver,
    key_value: Option<String>,
) -> Result<Service, Error> {
    match key_value {
        Some(key_value) => read_by_service_value(driver, &key_value).and_then(|key| {
            let key = key.ok_or_else(|| Error::Forbidden)?;
            let service_id = key.service_id.ok_or_else(|| Error::Forbidden)?;
            let service = driver
                .service_read_by_id(service_id)
                .map_err(Error::Driver)?;
            service.ok_or_else(|| Error::Forbidden)
        }),
        None => Err(Error::Forbidden),
    }
}

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

/// Create root key.
pub fn create_root(driver: &driver::Driver, name: &str) -> Result<Key, Error> {
    let value = uuid::Uuid::new_v4().to_simple().to_string();
    driver
        .key_create(name, &value, None, None)
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
        .key_create(name, &value, Some(service.id), user_id)
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

/// Read key by value (root only).
pub fn read_by_root_value(driver: &driver::Driver, value: &str) -> Result<Option<Key>, Error> {
    driver.key_read_by_root_value(value).map_err(Error::Driver)
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
