//! # Drivers
//! Binary application drivers.
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

use crate::core::{Audit, AuditMeta, Csrf, Key, Service, User};
#[cfg(feature = "postgres")]
pub use crate::driver::postgres::PostgresDriver;
#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::SqliteDriver;
use chrono::{DateTime, Utc};
use serde_json::Value;

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

    /// List audit logs where ID is less than.
    fn audit_list_where_id_lt(
        &self,
        lt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// List audit logs where ID is greater than.
    fn audit_list_where_id_gt(
        &self,
        gt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// List audit logs where created datetime is less than.
    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<&str>,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// List audit logs where created datetime is greater than.
    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<&str>,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// Create one audit log.
    fn audit_create(
        &self,
        meta: &AuditMeta,
        path: &str,
        data: &Value,
        key_id: Option<&str>,
        service_id: Option<&str>,
        user_id: Option<&str>,
        user_key_id: Option<&str>,
    ) -> Result<Audit, Error>;

    /// Read one audit log by ID.
    fn audit_read_by_id(&self, id: &str) -> Result<Option<Audit>, Error>;

    /// Delete many audit logs by created at time.
    fn audit_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> Result<usize, Error>;

    /// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        service_id: &str,
    ) -> Result<Csrf, Error>;

    /// Read one CSRF key, value pair.
    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, Error>;

    /// Delete one CSRF key, value pair.
    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, Error>;

    /// Delete many CSRF key, value pairs by time to live timestamp.
    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> Result<usize, Error>;

    /// List keys where ID is less than.
    fn key_list_where_id_lt(
        &self,
        lt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// List keys where ID is greater than.
    fn key_list_where_id_gt(
        &self,
        gt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error>;

    /// Create key.
    fn key_create(
        &self,
        is_enabled: bool,
        is_revoked: bool,
        name: &str,
        value: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<Key, Error>;

    /// Read key by ID.
    fn key_read_by_id(&self, id: &str) -> Result<Option<Key>, Error>;

    /// Read key by service and user ID.
    fn key_read_by_user_id(&self, service_id: &str, user_id: &str) -> Result<Option<Key>, Error>;

    /// Read key by root key value.
    fn key_read_by_root_value(&self, value: &str) -> Result<Option<Key>, Error>;

    /// Read key by service key value.
    fn key_read_by_service_value(&self, value: &str) -> Result<Option<Key>, Error>;

    /// Read key by service ID and user key value.
    fn key_read_by_user_value(&self, service_id: &str, value: &str) -> Result<Option<Key>, Error>;

    /// Update key by ID.
    fn key_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<Key, Error>;

    /// Update many keys by user ID.
    fn key_update_many_by_user_id(
        &self,
        user_id: &str,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<usize, Error>;

    /// Delete key by ID.
    fn key_delete_by_id(&self, id: &str) -> Result<usize, Error>;

    /// Delete root keys.
    fn key_delete_root(&self) -> Result<usize, Error>;

    /// List services where ID is less than.
    fn service_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, Error>;

    /// List services where ID is greater than.
    fn service_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, Error>;

    /// Create service.
    fn service_create(&self, is_enabled: bool, name: &str, url: &str) -> Result<Service, Error>;

    /// Read service by ID.
    fn service_read_by_id(&self, id: &str) -> Result<Option<Service>, Error>;

    /// Update service by ID.
    fn service_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<Service, Error>;

    /// Delete service by ID.
    fn service_delete_by_id(&self, id: &str) -> Result<usize, Error>;

    /// List users where ID is less than.
    fn user_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, Error>;

    /// List users where ID is greater than.
    fn user_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, Error>;

    /// List users where email is equal.
    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> Result<Vec<String>, Error>;

    /// Create user.
    fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, Error>;

    /// Read user by ID.
    fn user_read_by_id(&self, id: &str) -> Result<Option<User>, Error>;

    /// Read user by email address.
    fn user_read_by_email(&self, email: &str) -> Result<Option<User>, Error>;

    /// Update user by ID.
    fn user_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<User, Error>;

    /// Update user email by ID.
    fn user_update_email_by_id(&self, id: &str, email: &str) -> Result<usize, Error>;

    /// Update user password by ID.
    fn user_update_password_by_id(&self, id: &str, password_hash: &str) -> Result<usize, Error>;

    /// Delete user by ID.
    fn user_delete_by_id(&self, id: &str) -> Result<usize, Error>;
}

impl Clone for Box<Driver> {
    fn clone(&self) -> Box<Driver> {
        self.box_clone()
    }
}
