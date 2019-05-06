//! # Drivers
//! Binary application drivers.
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

use crate::core::{Csrf, Key, Service, User};
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

    /// Reset database (used for integration tests).
    fn reset(&self) -> Result<(), Error>;

    /// List keys where ID is less than.
    fn key_list_where_id_lt(&self, service_id: i64, lt: i64, limit: i64)
        -> Result<Vec<Key>, Error>;

    /// List keys where ID is greater than.
    fn key_list_where_id_gt(&self, service_id: i64, gt: i64, limit: i64)
        -> Result<Vec<Key>, Error>;

    /// Create key.
    fn key_create(
        &self,
        name: &str,
        value: &str,
        service_id: i64,
        user_id: Option<i64>,
    ) -> Result<Key, Error>;

    /// Read key by ID.
    fn key_read_by_id(&self, id: i64) -> Result<Option<Key>, Error>;

    /// Read key by service and user ID.
    fn key_read_by_user_id(&self, service_id: i64, user_id: i64) -> Result<Option<Key>, Error>;

    /// Read key by service key value.
    fn key_read_by_service_value(&self, value: &str) -> Result<Option<Key>, Error>;

    /// Read key by service ID and user key value.
    fn key_read_by_user_value(&self, service_id: i64, value: &str) -> Result<Option<Key>, Error>;

    /// Update key by ID.
    fn key_update_by_id(&self, id: i64, name: Option<&str>) -> Result<Key, Error>;

    /// Delete key by ID.
    fn key_delete_by_id(&self, id: i64) -> Result<usize, Error>;

    /// Create service.
    fn service_create(&self, name: &str, url: &str) -> Result<Service, Error>;

    /// Read service by ID.
    fn service_read_by_id(&self, id: i64) -> Result<Option<Service>, Error>;

    /// Update service by ID.
    fn service_update_by_id(&self, id: i64, name: Option<&str>) -> Result<Service, Error>;

    /// Delete service by ID.
    fn service_delete_by_id(&self, id: i64) -> Result<usize, Error>;

    /// List users where ID is less than.
    fn user_list_where_id_lt(&self, lt: i64, limit: i64) -> Result<Vec<User>, Error>;

    /// List users where ID is greater than.
    fn user_list_where_id_gt(&self, gt: i64, limit: i64) -> Result<Vec<User>, Error>;

    /// Create user.
    fn user_create(
        &self,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
        password_revision: Option<i64>,
    ) -> Result<User, Error>;

    /// Read user by ID.
    fn user_read_by_id(&self, id: i64) -> Result<Option<User>, Error>;

    /// Read user by email address.
    fn user_read_by_email(&self, email: &str) -> Result<Option<User>, Error>;

    /// Update user by ID.
    fn user_update_by_id(&self, id: i64, name: Option<&str>) -> Result<User, Error>;

    /// Update user password by ID.
    fn user_update_password_by_id(
        &self,
        id: i64,
        password_hash: &str,
        password_revision: i64,
    ) -> Result<usize, Error>;

    /// Delete user by ID.
    fn user_delete_by_id(&self, id: i64) -> Result<usize, Error>;

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
