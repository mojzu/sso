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
