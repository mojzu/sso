pub mod audit;
pub mod auth;
pub mod csrf;
pub mod jwt;
pub mod key;
pub mod metrics;
pub mod service;
pub mod user;

use crate::driver;
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use url::Url;

/// Default list limit.
pub const DEFAULT_LIMIT: i64 = 50;

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
    pub forwarded: Option<String>,
    pub path: String,
    pub data: Value,
    pub key_id: Option<String>,
    pub service_id: Option<String>,
    pub user_id: Option<String>,
    pub user_key_id: Option<String>,
}

/// Audit metadata, HTTP request information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMeta {
    user_agent: String,
    remote: String,
    forwarded: Option<String>,
}

impl AuditMeta {
    /// Create audit metadata from parameters.
    pub fn new<T1: Into<String>, T2: Into<Option<String>>>(
        user_agent: T1,
        remote: T1,
        forwarded: T2,
    ) -> Self {
        AuditMeta {
            user_agent: user_agent.into(),
            remote: remote.into(),
            forwarded: forwarded.into(),
        }
    }

    /// User agent string reference.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Remote IP string reference.
    pub fn remote(&self) -> &str {
        &self.remote
    }

    /// Forwarded for header optional string reference.
    pub fn forwarded(&self) -> Option<&str> {
        self.forwarded.as_ref().map(|x| &**x)
    }
}

/// Audit list query.
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditQuery {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub created_gte: Option<DateTime<Utc>>,
    pub created_lte: Option<DateTime<Utc>>,
    pub offset_id: Option<String>,
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

impl Service {
    pub fn callback_url<S: Serialize>(&self, type_: &str, data: S) -> Url {
        let mut url = Url::parse(&self.url).unwrap();
        let type_query = serde_urlencoded::to_string(&[("type", type_)]).unwrap();
        let data_query = serde_urlencoded::to_string(data).unwrap();
        let query = format!("{}&{}", type_query, data_query);
        url.set_query(Some(&query));
        url
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn builds_service_callback_url() {
        let service = Service {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            id: "6a9c6cfb7e15498b99e057153f0a212b".to_owned(),
            is_enabled: true,
            name: "Service Name".to_owned(),
            url: "http://localhost:9000".to_owned(),
        };
        let callback_data = &[
            ("email", "user@test.com"),
            ("token", "6a9c6cfb7e15498b99e057153f0a212b"),
        ];
        let url = service.callback_url("reset_password", callback_data);
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/?type=reset_password&email=user%40test.com&token=6a9c6cfb7e15498b99e057153f0a212b"
        );
    }
}
