#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "postgres")]
pub use crate::driver::postgres::DriverPostgres;
#[cfg(feature = "sqlite")]
pub use crate::driver::sqlite::DriverSqlite;

use crate::core::{
    Audit, AuditCreate, AuditList, CoreError, Csrf, CsrfCreate, Key, KeyCreate, KeyUpdate, Service,
    ServiceCreate, ServiceUpdate, User, UserCreate, UserUpdate,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Driver errors.
#[derive(Debug, Fail)]
pub enum DriverError {
    #[fail(display = "DriverError:LockFn {}", _0)]
    LockFn(String),

    #[fail(display = "DriverError:DieselResult {}", _0)]
    DieselResult(#[fail(cause)] diesel::result::Error),

    #[fail(display = "DriverError:DieselMigrations {}", _0)]
    DieselMigrations(#[fail(cause)] diesel_migrations::RunMigrationsError),

    #[fail(display = "DriverError:R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),
}

impl From<CoreError> for DriverError {
    fn from(e: CoreError) -> Self {
        Self::LockFn(format!("{}", e))
    }
}

impl From<diesel::result::Error> for DriverError {
    fn from(e: diesel::result::Error) -> Self {
        Self::DieselResult(e)
    }
}

impl From<diesel_migrations::RunMigrationsError> for DriverError {
    fn from(e: diesel_migrations::RunMigrationsError) -> Self {
        Self::DieselMigrations(e)
    }
}

impl From<r2d2::Error> for DriverError {
    fn from(e: r2d2::Error) -> Self {
        Self::R2d2(e)
    }
}

/// Driver result wrapper type.
pub type DriverResult<T> = Result<T, DriverError>;

/// Driver lock keys.
pub enum DriverLock {
    Transaction = 1,
}

/// Driver closure function type.
pub type DriverLockFn = Box<dyn FnOnce(&dyn DriverIf) -> DriverResult<usize>>;

/// Driver interface trait.
pub trait DriverIf {
    /// Run closure with an exclusive lock.
    fn exclusive_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<usize>;

    /// Run closure with a shared lock.
    fn shared_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<usize>;

    // ---------------
    // Audit Functions
    // ---------------

    /// List audit logs.
    fn audit_list(&self, list: &AuditList) -> DriverResult<Vec<Uuid>>;

    /// Create one audit log.
    fn audit_create(&self, data: &AuditCreate) -> DriverResult<Audit>;

    /// Read one audit log by ID.
    fn audit_read_by_id(
        &self,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Option<Audit>>;

    /// Read audit metrics, returns array of counts for distinct audit paths.
    fn audit_read_metrics(&self, service_id_mask: Option<Uuid>)
        -> DriverResult<Vec<(String, i64)>>;

    /// Delete many audit logs by created at time.
    fn audit_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> DriverResult<usize>;

    // --------------
    // CSRF Functions
    // --------------

    /// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf>;

    /// Read one CSRF key, value pair.
    fn csrf_read_by_key(&self, key: &str) -> DriverResult<Option<Csrf>>;

    /// Delete one CSRF key, value pair.
    fn csrf_delete_by_key(&self, key: &str) -> DriverResult<usize>;

    /// Delete many CSRF key, value pairs by time to live timestamp.
    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> DriverResult<usize>;

    // -------------
    // Key Functions
    // -------------

    /// List keys where ID is less than.
    fn key_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// List keys where ID is greater than.
    fn key_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>>;

    /// Create key.
    fn key_create(&self, create: &KeyCreate) -> DriverResult<Key>;

    /// Read key by ID.
    fn key_read_by_id(&self, id: Uuid) -> DriverResult<Option<Key>>;

    /// Read key by service and user ID.
    fn key_read_by_user_id(
        &self,
        service_id: Uuid,
        user_id: Uuid,
        is_enabled: bool,
        is_revoked: bool,
    ) -> DriverResult<Option<Key>>;

    /// Read key by root key value.
    fn key_read_by_root_value(&self, value: &str) -> DriverResult<Option<Key>>;

    /// Read key by service key value.
    fn key_read_by_service_value(&self, value: &str) -> DriverResult<Option<Key>>;

    /// Read key by service ID and user key value.
    fn key_read_by_user_value(&self, service_id: Uuid, value: &str) -> DriverResult<Option<Key>>;

    /// Update key by ID.
    fn key_update_by_id(&self, id: Uuid, update: &KeyUpdate) -> DriverResult<Key>;

    /// Update many keys by user ID.
    fn key_update_many_by_user_id(&self, user_id: Uuid, update: &KeyUpdate) -> DriverResult<usize>;

    /// Delete key by ID.
    fn key_delete_by_id(&self, id: Uuid) -> DriverResult<usize>;

    /// Delete root keys.
    fn key_delete_root(&self) -> DriverResult<usize>;

    // -----------------
    // Service Functions
    // -----------------

    /// List services where ID is less than.
    fn service_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// List services where ID is greater than.
    fn service_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// Create service.
    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service>;

    /// Read service by ID.
    fn service_read_by_id(&self, id: Uuid) -> DriverResult<Option<Service>>;

    /// Update service by ID.
    fn service_update_by_id(&self, id: Uuid, update: &ServiceUpdate) -> DriverResult<Service>;

    /// Delete service by ID.
    fn service_delete_by_id(&self, id: Uuid) -> DriverResult<usize>;

    // --------------
    // User Functions
    // --------------

    /// List users where ID is less than.
    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// List users where ID is greater than.
    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// List users where email is equal.
    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> DriverResult<Vec<Uuid>>;

    /// Create user.
    fn user_create(&self, create: &UserCreate) -> DriverResult<User>;

    /// Read user by ID.
    fn user_read_by_id(&self, id: Uuid) -> DriverResult<Option<User>>;

    /// Read user by email address.
    fn user_read_by_email(&self, email: &str) -> DriverResult<Option<User>>;

    /// Update user by ID.
    fn user_update_by_id(&self, id: Uuid, update: &UserUpdate) -> DriverResult<User>;

    /// Update user email by ID.
    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> DriverResult<usize>;

    /// Update user password by ID.
    fn user_update_password_by_id(&self, id: Uuid, password_hash: &str) -> DriverResult<usize>;

    /// Delete user by ID.
    fn user_delete_by_id(&self, id: Uuid) -> DriverResult<usize>;
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
