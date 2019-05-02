//! # Drivers
//! Binary application drivers.
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

/// Driver errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Diesel migrations error wrapper.
    #[fail(display = "DriverError::DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),
    /// R2d2 error wrapper.
    #[fail(display = "DriverError::R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),
}

/// Driver trait.
pub trait Driver: Send + Sync {}
