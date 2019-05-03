//! # Core
use crate::driver;
use chrono::{DateTime, Utc};

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Forbidden, authentication failure.
    #[fail(display = "CoreError::Forbidden")]
    Forbidden,
    /// Driver error wrapper.
    #[fail(display = "CoreError::DriverError {}", _0)]
    Driver(#[fail(cause)] driver::Error),
}

/// Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub url: String,
}

/// List services where ID is less than.
pub fn service_list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<Service>, Error> {
    unimplemented!();
}

/// List services where ID is greater than.
pub fn service_list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<Service>, Error> {
    unimplemented!();
}

/// Create service.
pub fn service_create(
    driver: &driver::Driver,
    name: &str,
    url: &str
) -> Result<Service, Error> {
    unimplemented!();
}

/// Read service by ID.
/// TODO(refactor): Use Option here?
pub fn service_read_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
) -> Result<Service, Error> {
    unimplemented!();
}

/// Update service by ID.
pub fn service_update_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<Service, Error> {
    unimplemented!();
}

/// Delete service by ID.
pub fn service_delete_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
) -> Result<usize, Error> {
    unimplemented!();
}

/// Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub value: String,
    pub service_id: i64,
    pub user_id: Option<i64>,
}

/// List keys where ID is less than.
pub fn key_list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<Key>, Error> {
    unimplemented!();
}

/// List keys where ID is greater than.
pub fn key_list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<Key>, Error> {
    unimplemented!();
}

/// Create key.
pub fn key_create(
    driver: &driver::Driver,
    service: &Service,
    name: &str,
    user_id: Option<i64>,
) -> Result<Key, Error> {
    unimplemented!();
}

/// Read key by ID.
/// TODO(refactor): Use Option here?
pub fn key_read_by_id(driver: &driver::Driver, service: &Service, id: i64) -> Result<Key, Error> {
    unimplemented!();
}

/// Update key by ID.
pub fn key_update_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<Key, Error> {
    unimplemented!();
}

/// Delete key by ID.
pub fn key_delete_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
) -> Result<usize, Error> {
    unimplemented!();
}

/// User.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub email: String,
}

/// Authenticate service key and return associated service.
pub fn service_authenticate(
    driver: &driver::Driver,
    key_value: Option<String>,
) -> Result<Service, Error> {
    match key_value {
        Some(key_value) => {
            driver
                .service_read_by_key_value(&key_value)
                .map_err(|error| match error {
                    driver::Error::NotFound => Error::Forbidden,
                    _ => Error::Driver(error),
                })
        }
        None => Err(Error::Forbidden),
    }
}

/// List users where ID is less than.
pub fn user_list_where_id_lt(
    driver: &driver::Driver,
    service: &Service,
    lt: i64,
    limit: i64,
) -> Result<Vec<User>, Error> {
    unimplemented!();
}

/// List users where ID is greater than.
pub fn user_list_where_id_gt(
    driver: &driver::Driver,
    service: &Service,
    gt: i64,
    limit: i64,
) -> Result<Vec<User>, Error> {
    unimplemented!();
}

/// Create user.
pub fn user_create(
    driver: &driver::Driver,
    service: &Service,
    name: &str,
    email: &str,
    password: Option<&str>,
) -> Result<User, Error> {
    unimplemented!();
}

/// Read user by ID.
/// TODO(refactor): Use Option here?
pub fn user_read_by_id(driver: &driver::Driver, service: &Service, id: i64) -> Result<User, Error> {
    unimplemented!();
}

/// Update user by ID.
pub fn user_update_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
    name: Option<&str>,
) -> Result<User, Error> {
    unimplemented!();
}

/// Delete user by ID.
pub fn user_delete_by_id(
    driver: &driver::Driver,
    service: &Service,
    id: i64,
) -> Result<usize, Error> {
    unimplemented!();
}

/// User token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub user_id: i64,
    pub token: String,
    pub token_expires: usize,
}

/// User key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKey {
    pub user_id: i64,
    pub key: String,
}

/// User authentication using email address and password.
pub fn auth_login(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
    password: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// User reset password request.
pub fn auth_reset_password(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// User reset password confirm.
pub fn auth_reset_password_confirm(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// Verify user key.
pub fn auth_key_verify(
    driver: &driver::Driver,
    service: &Service,
    key: &str,
) -> Result<UserKey, Error> {
    unimplemented!();
}

/// Revoke user key.
pub fn auth_key_revoke(
    driver: &driver::Driver,
    service: &Service,
    key: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// Verify user token.
pub fn auth_token_verify(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// Refresh user token.
pub fn auth_token_refresh(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// Revoke user token.
pub fn auth_token_revoke(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &driver::Driver,
    service_id: i64,
    email: &str,
) -> Result<Service, UserToken> {
    unimplemented!();
}

// TODO(refactor): Refactor this.
// pub fn auth_reset_password() {
// .and_then(|(service, (user, token_response))| {
//     // Send user email with reset password confirmation link.
//     match email::send_reset_password(
//         data.smtp(),
//         &user,
//         &service,
//         &token_response.token,
//     ) {
//         Ok(_) => Ok(token_response),
//         // Log warning in case of failure to send email.
//         Err(e) => {
//             warn!("Failed to send reset password email ({})", e);
//             Ok(token_response)
//         }
//     }
// })
// }

// pub fn oauth2_login(
//     data: &web::Data<ApiData>,
//     email: &str,
//     service_id: i64,
// ) -> Result<(TokenData, AuthService), ApiError> {
//     let token = data
//         .db
//         .oauth2_login(email, service_id)
//         .map_err(ApiError::Db)?;
//     let service = data
//         .db
//         .service_read_by_id(service_id, service_id)
//         .map_err(ApiError::Db)?;
//     Ok((token, service))
// }
