//! # Drivers
//! Binary application drivers.
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use crate::core::Service;

/// Driver errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Not found.
    #[fail(display = "DriverError::NotFound")]
    NotFound,
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
    fn service_read_by_key_value(&self, key_value: &str) -> Result<Service, Error>;
}

impl Clone for Box<Driver> {
    fn clone(&self) -> Box<Driver> {
        self.box_clone()
    }
}
