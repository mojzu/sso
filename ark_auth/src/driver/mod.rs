//! # Drivers
//! Binary application drivers.
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use crate::core::{Csrf, Service};
use chrono::{DateTime, Utc};

/// Driver errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Diesel result error wrapper.
    #[fail(display = "DriverError::Diesel {}", _0)]
    Diesel(#[fail(cause)] diesel::result::Error),
    /// Diesel migrations error wrapper.
    #[fail(display = "DriverError::DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),
    /// R2d2 error wrapper.
    #[fail(display = "DriverError::R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),
}

/// Driver trait.
pub trait Driver: Send + Sync {
    /// Return a boxed trait containing clone of self.
    fn box_clone(&self) -> Box<Driver>;

    /// Read service by unique key value.
    fn service_read_by_key_value(&self, key_value: &str) -> Result<Option<Service>, Error>;

    /// Create one CSRF key, value pair. Key must be unique.
    fn csrf_create(&self, key: &str, value: &str, service_id: i64) -> Result<Csrf, Error>;

    /// Read one CSRF key, value pair.
    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, Error>;

    /// Delete one CSRF key, value pair.
    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, Error>;

    /// Delete many CSRF key, value pairs by created at time.
    fn csrf_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> Result<usize, Error>;
}

impl Clone for Box<Driver> {
    fn clone(&self) -> Box<Driver> {
        self.box_clone()
    }
}
