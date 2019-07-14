//! # Drivers
//! Binary application drivers.
#[cfg(feature = "file")]
mod file;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "file")]
pub use crate::driver::file::FileDriver;
#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::SqliteDriver;

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
}

impl Clone for Box<Driver> {
    fn clone(&self) -> Box<Driver> {
        self.box_clone()
    }
}
