//! # Drivers
//! Binary application drivers.
// #[cfg(feature = "file")]
// mod file;
#[cfg(feature = "sqlite")]
mod sqlite;

// TODO(feature): Implement file driver (ark_ota support?).
// #[cfg(feature = "file")]
// pub use crate::driver::file::FileDriver;
use crate::core::{Data, Disk, DiskOptions, Key, Version};
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

    /// Read disk by ID.
    fn disk_read_by_id(&self, id: &str) -> Result<Option<Disk>, Error>;

    /// Read disk by name.
    fn disk_read_by_name(&self, name: &str) -> Result<Option<Disk>, Error>;

    /// Create key.
    fn key_create(&self, name: &str, disk_id: &str) -> Result<Key, Error>;

    /// Read key by ID.
    fn key_read_by_id(&self, id: &str) -> Result<Option<Key>, Error>;

    /// Read key by name.
    fn key_read_by_name(&self, name: &str, disk_id: &str) -> Result<Option<Key>, Error>;

    /// Update key by ID.
    fn key_update_by_id(
        &self,
        id: &str,
        name: Option<&str>,
        version_id: Option<&str>,
    ) -> Result<usize, Error>;

    /// Create version.
    fn version_create(&self, hash: &[u8], size: i64, key_id: &str) -> Result<Version, Error>;

    /// Read version by ID.
    fn version_read_by_id(&self, id: &str) -> Result<Option<Version>, Error>;

    /// Create data.
    fn data_create(&self, chunk: i64, data: &[u8], version_id: &str) -> Result<Data, Error>;

    /// Read data by chunk.
    fn data_read_by_chunk(&self, chunk: i64, version_id: &str) -> Result<Option<Data>, Error>;
}

impl Clone for Box<Driver> {
    fn clone(&self) -> Box<Driver> {
        self.box_clone()
    }
}
