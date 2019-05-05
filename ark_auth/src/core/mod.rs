pub mod auth;
pub mod csrf;
pub mod key;
pub mod service;
pub mod user;

use crate::driver;
use chrono::{DateTime, Utc};

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Forbidden, authentication failure.
    #[fail(display = "CoreError::Forbidden")]
    Forbidden,
    /// Invalid password, password is incorrect or null.
    #[fail(display = "CoreError::PasswordInvalid")]
    PasswordInvalid,

    /// Driver error wrapper.
    #[fail(display = "CoreError::DriverError {}", _0)]
    Driver(#[fail(cause)] driver::Error),
    /// Bcrypt error wrapper.
    #[fail(display = "DbError::Bcrypt {}", _0)]
    Bcrypt(#[fail(cause)] bcrypt::BcryptError),
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

/// User.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
    #[serde(skip)]
    pub password_revision: Option<i64>,
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

/// CSRF.
pub struct Csrf {
    pub created_at: DateTime<Utc>,
    pub key: String,
    pub value: String,
    pub service_id: i64,
}

/// Hash password string using bcrypt, none is returned for none as input.
pub fn hash_password(password: Option<&str>) -> Result<Option<String>, Error> {
    match password {
        Some(password) => {
            let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(Error::Bcrypt)?;
            Ok(Some(hashed))
        }
        None => Ok(None),
    }
}

/// Check if password string and password bcrypt hash are equal, error is returned for none as user password.
pub fn check_password(user_password: Option<&str>, check_password: &str) -> Result<(), Error> {
    match user_password {
        Some(user_password) => bcrypt::verify(check_password, user_password)
            .map_err(Error::Bcrypt)
            .and_then(|verified| {
                if verified {
                    Ok(())
                } else {
                    Err(Error::PasswordInvalid)
                }
            }),
        None => Err(Error::PasswordInvalid),
    }
}
