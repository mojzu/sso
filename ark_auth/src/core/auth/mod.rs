pub mod oauth2;

use crate::core::{Error, Service, UserKey, UserToken};
use crate::driver;

/// User authentication using email address and password.
pub fn login(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
    password: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// User reset password request.
pub fn reset_password(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// User reset password confirm.
pub fn reset_password_confirm(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// Verify user key.
pub fn key_verify(driver: &driver::Driver, service: &Service, key: &str) -> Result<UserKey, Error> {
    unimplemented!();
}

/// Revoke user key.
pub fn key_revoke(driver: &driver::Driver, service: &Service, key: &str) -> Result<usize, Error> {
    unimplemented!();
}

/// Verify user token.
pub fn token_verify(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// Refresh user token.
pub fn token_refresh(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// Revoke user token.
pub fn token_revoke(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

// TODO(refactor): Refactor this.
// pub fn reset_password() {
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

// Map invalid password, not found errors to bad request to prevent leakage.
// TODO(feature): Warning logs for bad requests.
// TODO(refactor): Refactor this.
// DbError::InvalidPassword | DbError::NotFound => ApiError::BadRequest,
