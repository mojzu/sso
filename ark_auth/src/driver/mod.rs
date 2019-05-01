//! # Drivers
//! Binary application drivers.
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub mod postgres;
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
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
pub trait Driver {}
