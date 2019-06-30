pub mod audit;
pub mod auth;
pub mod csrf;
pub mod jwt;
pub mod key;
pub mod service;
pub mod user;

use crate::driver;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Bad request.
    #[fail(display = "CoreError::BadRequest")]
    BadRequest,
    /// Forbidden, authentication failure.
    #[fail(display = "CoreError::Forbidden")]
    Forbidden,
    /// Driver error wrapper.
    #[fail(display = "CoreError::Driver {}", _0)]
    Driver(#[fail(cause)] driver::Error),
    /// Bcrypt error wrapper.
    #[fail(display = "CoreError::Bcrypt {}", _0)]
    Bcrypt(#[fail(cause)] bcrypt::BcryptError),
    /// JSON web token error wrapper.
    #[fail(display = "CoreError::Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),
}

/// Audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audit {
    pub created_at: DateTime<Utc>,
    pub id: String,
    pub user_agent: String,
    pub remote: String,
    pub forwarded_for: Option<String>,
    pub path: String,
    pub data: Value,
    pub key_id: Option<String>,
    pub service_id: Option<String>,
    pub user_id: Option<String>,
    pub user_key_id: Option<String>,
}

/// Audit meta.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMeta {
    pub user_agent: String,
    pub remote: String,
    pub forwarded_for: Option<String>,
}

/// Audit list query.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditQuery {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub limit: Option<i64>,
}

/// CSRF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Csrf {
    pub created_at: DateTime<Utc>,
    pub key: String,
    pub value: String,
    pub ttl: DateTime<Utc>,
    pub service_id: String,
}

/// Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub name: String,
    pub value: String,
    pub service_id: Option<String>,
    pub user_id: Option<String>,
}

/// Key query.
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyQuery {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub limit: Option<i64>,
}

/// Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub is_enabled: bool,
    pub name: String,
    pub url: String,
}

/// Service query.
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceQuery {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub limit: Option<i64>,
}

/// User.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: String,
    pub is_enabled: bool,
    pub name: String,
    pub email: String,
    #[serde(skip)]
    pub password_hash: Option<String>,
}

/// User query.
#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuery {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub limit: Option<i64>,
    pub email_eq: Option<String>,
}

/// User token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserToken {
    pub user_id: String,
    pub access_token: String,
    pub access_token_expires: i64,
    pub refresh_token: String,
    pub refresh_token_expires: i64,
}

/// Partial user token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTokenPartial {
    pub user_id: String,
    pub access_token: String,
    pub access_token_expires: i64,
}

/// User key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserKey {
    pub user_id: String,
    pub key: String,
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
pub fn check_password(password_hash: Option<&str>, password: &str) -> Result<(), Error> {
    match password_hash {
        Some(password_hash) => bcrypt::verify(password, password_hash)
            .map_err(Error::Bcrypt)
            .and_then(|verified| {
                if verified {
                    Ok(())
                } else {
                    Err(Error::BadRequest)
                }
            }),
        None => Err(Error::BadRequest),
    }
}
