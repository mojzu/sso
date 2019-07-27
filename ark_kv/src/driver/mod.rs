//! # Drivers
//! Binary application drivers.
// #[cfg(feature = "file")]
// mod file;
#[cfg(feature = "sqlite")]
mod sqlite;

// TODO(feature): Implement file driver (ark_ota support?).
// #[cfg(feature = "file")]
// pub use crate::driver::file::FileDriver;
use crate::core::{Disk, DiskOptions};
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

    /// List disks where name is greater than.
    fn disk_list_where_name_gte(
        &self,
        name_gte: &str,
        offset_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<String>, Error>;

    /// Create disk.
    fn disk_create(&self, name: &str, options: &DiskOptions) -> Result<Disk, Error>;

    /// Read disk by ID, returns Err in case disk does not exist.
    fn disk_read_by_id(&self, id: &str) -> Result<Disk, Error>;

    /// Read disk by name, returns Err in case disk does not exist.
    fn disk_read_by_name(&self, name: &str) -> Result<Disk, Error>;
}

impl Clone for Box<Driver> {
    fn clone(&self) -> Box<Driver> {
        self.box_clone()
    }
}
