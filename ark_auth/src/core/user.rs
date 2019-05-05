use crate::core::{hash_password, Error, Service, User};
use crate::driver;

/// List users where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    _service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<User>, Error> {
    driver
        .user_list_where_id_lt(lt, limit)
        .map_err(Error::Driver)
}

/// List users where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    _service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<User>, Error> {
    driver
        .user_list_where_id_gt(gt, limit)
        .map_err(Error::Driver)
}

/// Create user.
pub fn create(
    driver: &driver::Driver,
    _service: &Service,
    name: &str,
    email: &str,
    password: Option<&str>,
) -> Result<User, Error> {
    let password_hash = hash_password(password)?;
    let password_revision = match password_hash {
        Some(_) => Some(1),
        None => None,
    };
    driver
        .user_create(
            name,
            email,
            password_hash.as_ref().map(|x| &**x),
            password_revision,
        )
        .map_err(Error::Driver)
}

/// Read user by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service: &Service,
    id: i64,
) -> Result<Option<User>, Error> {
    driver.user_read_by_id(id).map_err(Error::Driver)
}

/// Update user by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<User, Error> {
    driver.user_update_by_id(id, name).map_err(Error::Driver)
}

/// Delete user by ID.
pub fn delete_by_id(driver: &driver::Driver, _service: &Service, id: i64) -> Result<usize, Error> {
    driver.user_delete_by_id(id).map_err(Error::Driver)
}
