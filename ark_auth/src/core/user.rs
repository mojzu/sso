use crate::core::{Error, Service, User};
use crate::driver;

/// List users where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<User>, Error> {
    unimplemented!();
}

/// List users where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<User>, Error> {
    unimplemented!();
}

/// Create user.
pub fn create(
    driver: &driver::Driver,
    service: &Service,
    name: &str,
    email: &str,
    password: Option<&str>,
) -> Result<User, Error> {
    unimplemented!();
}

/// Read user by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
) -> Result<Option<User>, Error> {
    unimplemented!();
}

/// Update user by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<User, Error> {
    unimplemented!();
}

/// Delete user by ID.
pub fn delete_by_id(driver: &driver::Driver, service: &Service, id: i64) -> Result<usize, Error> {
    unimplemented!();
}
