use crate::core::audit::AuditBuilder;
use crate::core::{hash_password, Error, Service, User, UserQuery};
use crate::driver;

/// List users using query.
pub fn list(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    query: &UserQuery,
) -> Result<Vec<String>, Error> {
    unimplemented!();
}

/// List users where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    lt: &str,
    limit: i64,
) -> Result<Vec<String>, Error> {
    driver
        .user_list_where_id_lt(lt, limit)
        .map_err(Error::Driver)
}

/// List users where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    gt: &str,
    limit: i64,
) -> Result<Vec<String>, Error> {
    driver
        .user_list_where_id_gt(gt, limit)
        .map_err(Error::Driver)
}

/// List users where email is equal.
pub fn list_where_email_eq(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    email_eq: &str,
    limit: i64,
) -> Result<Vec<String>, Error> {
    driver
        .user_list_where_email_eq(email_eq, limit)
        .map_err(Error::Driver)
}

/// Create user.
/// Returns bad request if email address is not unique.
pub fn create(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    is_enabled: bool,
    name: &str,
    email: &str,
    password: Option<&str>,
) -> Result<User, Error> {
    let user = read_by_email(driver, service_mask, email)?;
    if user.is_some() {
        return Err(Error::BadRequest);
    }

    let password_hash = hash_password(password)?;
    driver
        .user_create(
            is_enabled,
            name,
            email,
            password_hash.as_ref().map(|x| &**x),
        )
        .map_err(Error::Driver)
}

/// Read user by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
) -> Result<Option<User>, Error> {
    driver.user_read_by_id(id).map_err(Error::Driver)
}

/// Read user by email.
pub fn read_by_email(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    email: &str,
) -> Result<Option<User>, Error> {
    driver.user_read_by_email(email).map_err(Error::Driver)
}

/// Update user by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
    is_enabled: Option<bool>,
    name: Option<&str>,
) -> Result<User, Error> {
    driver
        .user_update_by_id(id, is_enabled, name)
        .map_err(Error::Driver)
}

/// Update user email by ID.
pub fn update_email_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: &str,
    email: &str,
) -> Result<usize, Error> {
    driver
        .user_update_email_by_id(id, email)
        .map_err(Error::Driver)
}

/// Update user password by ID.
pub fn update_password_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: &str,
    password: &str,
) -> Result<usize, Error> {
    let password_hash = hash_password(Some(password))?.ok_or_else(|| Error::Forbidden)?;
    driver
        .user_update_password_by_id(id, &password_hash)
        .map_err(Error::Driver)
}

/// Delete user by ID.
pub fn delete_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
) -> Result<usize, Error> {
    driver.user_delete_by_id(id).map_err(Error::Driver)
}
