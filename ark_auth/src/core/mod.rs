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
