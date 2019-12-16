mod audit;
mod csrf;
pub mod env;
mod error;
mod jwt;
mod key;
mod metrics;
pub mod pattern;
mod service;
mod template;
mod user;

#[cfg(feature = "postgres")]
mod postgres;

#[cfg(feature = "postgres")]
pub use crate::driver::postgres::DriverPostgres;
pub use crate::driver::{
    audit::*, csrf::*, error::*, jwt::*, key::*, metrics::*, service::*, template::*, user::*,
};

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Default limit.
pub const DEFAULT_LIMIT: i64 = 50;

/// Default CSRF expires seconds.
pub const DEFAULT_CSRF_EXPIRES_S: i64 = 1000;

/// Authorisation header name.
pub const HEADER_AUTHORISATION_NAME: &str = "Authorization";

/// User authorisation header name.
pub const HEADER_USER_AUTHORISATION_NAME: &str = "User-Authorization";

/// Driver closure function type.
pub type DriverLockFn = Box<dyn FnOnce(&dyn DriverIf) -> DriverResult<bool>>;

/// Driver interface trait.
pub trait DriverIf {
    /// Run closure with an exclusive lock.
    fn exclusive_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<bool>;

    /// Run closure with a shared lock.
    fn shared_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<bool>;

    // ---------------
    // Audit Functions
    // ---------------

    /// List audit logs.
    fn audit_list(&self, list: &AuditList) -> DriverResult<Vec<Audit>>;

    /// Create audit log.
    fn audit_create(&self, data: &AuditCreate) -> DriverResult<Audit>;

    /// Read audit log.
    fn audit_read(&self, read: &AuditRead) -> DriverResult<Option<Audit>>;

    /// Read audit metrics, returns array of counts for distinct audit types.
    fn audit_read_metrics(
        &self,
        from: &DateTime<Utc>,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, u16, i64)>>;

    /// Update audit log, append data to data array.
    fn audit_update(
        &self,
        id: &Uuid,
        update: &AuditUpdate,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Audit>;

    /// Delete many audit logs.
    fn audit_delete(&self, created_at: &DateTime<Utc>) -> DriverResult<usize>;

    // --------------
    // CSRF Functions
    // --------------

    /// Create CSRF token.
    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf>;

    /// Read CSRF token. CSRF token is deleted after one read.
    fn csrf_read(&self, key: &str) -> DriverResult<Option<Csrf>>;

    // -------------
    // Key Functions
    // -------------

    /// List keys.
    fn key_list(&self, list: &KeyList) -> DriverResult<Vec<Key>>;

    /// Count keys.
    fn key_count(&self, count: &KeyCount) -> DriverResult<usize>;

    /// Create key.
    ///
    /// Returns error if more than one `Token` or `Totp` type would be enabled for user keys.
    /// Returns error if related service or user does not exist.
    fn key_create(&self, create: &KeyCreate) -> DriverResult<KeyWithValue>;

    /// Read key.
    fn key_read(&self, read: &KeyRead) -> DriverResult<Option<KeyWithValue>>;

    /// Update key.
    fn key_update(&self, id: &Uuid, update: &KeyUpdate) -> DriverResult<Key>;

    /// Update many keys by user ID.
    fn key_update_many(&self, user_id: &Uuid, update: &KeyUpdate) -> DriverResult<usize>;

    /// Delete key.
    fn key_delete(&self, id: &Uuid) -> DriverResult<usize>;

    // -----------------
    // Service Functions
    // -----------------

    /// List services.
    fn service_list(&self, list: &ServiceList) -> DriverResult<Vec<Service>>;

    /// Create service.
    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service>;

    /// Read service.
    fn service_read(&self, read: &ServiceRead) -> DriverResult<Option<Service>>;

    /// Update service.
    fn service_update(&self, id: &Uuid, update: &ServiceUpdate) -> DriverResult<Service>;

    /// Delete service.
    fn service_delete(&self, id: &Uuid) -> DriverResult<usize>;

    // --------------
    // User Functions
    // --------------

    /// List users.
    fn user_list(&self, list: &UserList) -> DriverResult<Vec<User>>;

    /// Create user.
    ///
    /// Returns error if email address is not unique.
    fn user_create(&self, create: &UserCreate) -> DriverResult<User>;

    /// Read user.
    fn user_read(&self, read: &UserRead) -> DriverResult<Option<User>>;

    /// Update user.
    fn user_update(&self, id: &Uuid, update: &UserUpdate) -> DriverResult<User>;

    /// Delete user.
    fn user_delete(&self, id: &Uuid) -> DriverResult<usize>;
}

/// Driver trait.
pub trait Driver: DriverIf + Send + Sync {
    /// Return a boxed trait containing clone of self.
    fn box_clone(&self) -> Box<dyn Driver>;

    /// Return a reference to driver interface.
    fn as_if(&self) -> &dyn DriverIf;
}

impl Clone for Box<dyn Driver> {
    fn clone(&self) -> Box<dyn Driver> {
        self.box_clone()
    }
}
