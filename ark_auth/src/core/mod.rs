pub mod auth;
pub mod csrf;
pub mod key;
pub mod service;
pub mod user;

use crate::driver;
use chrono::{DateTime, Utc};

// TODO(refactor): Error string descriptions for logs.

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
    #[fail(display = "DbError::Bcrypt {}", _0)]
    Bcrypt(#[fail(cause)] bcrypt::BcryptError),
    /// JSON web token error wrapper.
    #[fail(display = "DbError::Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),
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
    pub service_id: Option<i64>,
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

// // TODO(test): Reimplement as unit test(s).
// pub fn password_confirm_test(driver: &Driver, app: &mut TestServerRuntime) {
//     let (service, key) = support::service_key(driver);
//     let (_servic2, key2) = support::service_key(driver);
//     let (user, _key) = support::user_key(driver, &service, Some("guest"));
//     let (_, token) = core::auth::reset_password(driver, &service, &user.email).unwrap();
//     // Service 2 cannot confirm reset password.
//     // 400 BAD REQUEST response.
//     let payload = format!(
//         r#"{{"token": "{}", "password": "guestguest"}}"#,
//         &token.token
//     );
//     let (status_code, content_length, bytes) = support::app_post(
//         app,
//         "/v1/auth/reset/password/confirm",
//         Some(&key2.value),
//         payload,
//     );
//     assert_eq!(status_code, StatusCode::BAD_REQUEST);
//     assert_eq!(content_length, 0);
//     assert_eq!(bytes.len(), 0);
//     // Confirm reset password success.
//     // 200 OK response.
//     let payload = format!(
//         r#"{{"token": "{}", "password": "guestguest"}}"#,
//         &token.token
//     );
//     let (status_code, content_length, bytes) = support::app_post(
//         app,
//         "/v1/auth/reset/password/confirm",
//         Some(&key.value),
//         payload,
//     );
//     assert_eq!(status_code, StatusCode::OK);
//     assert_eq!(content_length, bytes.len());
//     let body: server::route::auth::reset::PasswordConfirmResponse =
//         serde_json::from_slice(&bytes).unwrap();
//     assert!(body.meta.password_strength.is_some());
//     assert_eq!(body.meta.password_pwned, None);
//     // User password is updated.
//     // 200 OK response.
//     let payload = format!(
//         r#"{{"email": "{}", "password": "guestguest"}}"#,
//         &user.email
//     );
//     let (status_code, _content_length, _bytes) =
//         support::app_post(app, "/v1/auth/login", Some(&key.value), payload);
//     assert_eq!(status_code, StatusCode::OK);
//     // Cannot reuse token.
//     // 400 BAD REQUEST response.
//     let payload = format!(r#"{{"token": "{}", "password": "guest"}}"#, &token.token);
//     let (status_code, content_length, bytes) = support::app_post(
//         app,
//         "/v1/auth/reset/password/confirm",
//         Some(&key.value),
//         payload,
//     );
//     assert_eq!(status_code, StatusCode::BAD_REQUEST);
//     assert_eq!(content_length, 0);
//     assert_eq!(bytes.len(), 0);
// }
