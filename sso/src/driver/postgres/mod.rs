mod diesel_admin;
mod model;
mod schema;

use crate::{
    driver::{
        env,
        postgres::model::{ModelAudit, ModelCsrf, ModelKey, ModelService, ModelUser},
    },
    Audit, AuditCreate, AuditList, AuditRead, AuditUpdate, Csrf, CsrfCreate, DriverError,
    DriverResult, Key, KeyCount, KeyCreate, KeyList, KeyRead, KeyUpdate, KeyWithValue, Service,
    ServiceCreate, ServiceList, ServiceRead, ServiceUpdate, User, UserCreate, UserList, UserRead,
    UserUpdate,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, r2d2::ConnectionManager};
use std::fmt;
use uuid::Uuid;

embed_migrations!("migrations");

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Driver for PostgreSQL.
#[derive(Clone)]
pub struct Postgres {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl fmt::Debug for Postgres {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Postgres {{ pool }}")
    }
}

/// Driver closure function type.
pub type PostgresLockFn = Box<dyn FnOnce(&PgConnection) -> DriverResult<bool>>;

impl Postgres {
    /// Initialise driver with connection URL and number of pooled connections.
    pub fn initialise(url: &str, connections: Option<u32>) -> DriverResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let mut pool = r2d2::Pool::builder();
        if let Some(connections) = connections {
            pool = pool.max_size(connections);
        }
        let pool = pool.build(manager).map_err(DriverError::R2d2)?;
        let driver = Postgres { pool };
        driver.run_migrations()?;
        Ok(driver)
    }

    pub fn from_env<U, M>(url_name: U, connections_name: M) -> Self
    where
        U: AsRef<str>,
        M: AsRef<str>,
    {
        let url = env::string(url_name.as_ref())
            .expect("Failed to read postgres URL environment variable.");
        let connections = env::value_opt::<u32>(connections_name.as_ref())
            .expect("Failed to read postgres connections environment variable.");
        Self::initialise(&url, connections).expect("Failed to initialise postgres connection.")
    }

    fn conn(&self) -> DriverResult<PooledConnection> {
        self.pool.get().map_err(DriverError::R2d2)
    }

    fn run_migrations(&self) -> DriverResult<()> {
        let connection = self.conn()?;
        embedded_migrations::run(&connection).map_err(DriverError::DieselMigrations)
    }
}

impl Postgres {
    /// Run closure with an exclusive lock.
    pub fn exclusive_lock(&self, key: i32, func: PostgresLockFn) -> DriverResult<bool> {
        use diesel_admin::*;

        let conn = self.conn()?;
        conn.transaction(|| {
            if diesel::select(pg_try_advisory_xact_lock(1, key as i32)).get_result::<bool>(&conn)? {
                func(&conn)
            } else {
                Err(DriverError::Locked(key))
            }
        })
    }

    /// Run closure with a shared lock.
    pub fn shared_lock(&self, key: i32, func: PostgresLockFn) -> DriverResult<bool> {
        use diesel_admin::*;

        let conn = self.conn()?;
        conn.transaction(|| {
            if diesel::select(pg_try_advisory_xact_lock_shared(1, key as i32))
                .get_result::<bool>(&conn)?
            {
                func(&conn)
            } else {
                Err(DriverError::Locked(key))
            }
        })
    }

    // ---------------
    // Audit Functions
    // ---------------

    /// List audit logs.
    pub fn audit_list(
        &self,
        list: &AuditList,
        service_id: Option<Uuid>,
    ) -> DriverResult<Vec<Audit>> {
        let conn = self.conn()?;
        ModelAudit::list(&conn, list, service_id)
    }

    /// Create audit log.
    pub fn audit_create(&self, create: &AuditCreate) -> DriverResult<Audit> {
        let conn = self.conn()?;
        ModelAudit::create(&conn, create)
    }

    /// Read audit log.
    pub fn audit_read(
        &self,
        read: &AuditRead,
        service_id: Option<Uuid>,
    ) -> DriverResult<Option<Audit>> {
        let conn = self.conn()?;
        ModelAudit::read(&conn, read, service_id)
    }

    /// Read audit metrics, returns array of counts for distinct audit types.
    pub fn audit_read_metrics(
        &self,
        from: &DateTime<Utc>,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, u16, i64)>> {
        let conn = self.conn()?;
        ModelAudit::read_metrics(&conn, from, service_id_mask)
    }

    /// Update audit log, append data to data array.
    pub fn audit_update(
        &self,
        update: &AuditUpdate,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Audit> {
        let conn = self.conn()?;
        ModelAudit::update(&conn, update, service_id_mask)
    }

    /// Delete many audit logs.
    pub fn audit_delete(&self, created_at: &DateTime<Utc>) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelAudit::delete(&conn, created_at)
    }

    // --------------
    // CSRF Functions
    // --------------

    /// Create CSRF token.
    pub fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf> {
        let conn = self.conn()?;
        ModelCsrf::create(&conn, create)
    }

    /// Read CSRF token. CSRF token is deleted after one read.
    pub fn csrf_read(&self, key: &str) -> DriverResult<Option<Csrf>> {
        let conn = self.conn()?;
        ModelCsrf::read(&conn, key)
    }

    // -------------
    // Key Functions
    // -------------

    /// List keys.
    pub fn key_list(&self, list: &KeyList, service_id: Option<Uuid>) -> DriverResult<Vec<Key>> {
        let conn = self.conn()?;
        ModelKey::list(&conn, list, service_id)
    }

    /// Count keys.
    pub fn key_count(&self, count: &KeyCount) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::count(&conn, count)
    }

    /// Create key.
    ///
    /// Returns error if more than one `Token` or `Totp` type would be enabled for user keys.
    /// Returns error if related service or user does not exist.
    pub fn key_create(&self, create: &KeyCreate) -> DriverResult<KeyWithValue> {
        let conn = self.conn()?;
        ModelKey::create(&conn, create)
    }

    /// Read key.
    pub fn key_read(
        &self,
        read: &KeyRead,
        service_id: Option<Uuid>,
    ) -> DriverResult<Option<KeyWithValue>> {
        let conn = self.conn()?;
        ModelKey::read(&conn, read, service_id)
    }

    /// Update key.
    pub fn key_update(&self, update: &KeyUpdate) -> DriverResult<Key> {
        let conn = self.conn()?;
        ModelKey::update(&conn, update)
    }

    /// Update many keys by user ID.
    pub fn key_update_many(&self, user_id: &Uuid, update: &KeyUpdate) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::update_many(&conn, user_id, update)
    }

    /// Delete key.
    pub fn key_delete(&self, id: &Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::delete(&conn, id)
    }

    // -----------------
    // Service Functions
    // -----------------

    /// List services.
    pub fn service_list(&self, list: &ServiceList) -> DriverResult<Vec<Service>> {
        let conn = self.conn()?;
        ModelService::list(&conn, list)
    }

    /// Create service.
    pub fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service> {
        let conn = self.conn()?;
        ModelService::create(&conn, create)
    }

    /// Read service.
    pub fn service_read(
        &self,
        read: &ServiceRead,
        service_id: Option<Uuid>,
    ) -> DriverResult<Option<Service>> {
        let conn = self.conn()?;
        ModelService::read(&conn, read, service_id)
    }

    /// Update service.
    pub fn service_update(&self, update: &ServiceUpdate) -> DriverResult<Service> {
        let conn = self.conn()?;
        ModelService::update(&conn, update)
    }

    /// Delete service.
    pub fn service_delete(&self, id: &Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelService::delete(&conn, id)
    }

    // --------------
    // User Functions
    // --------------

    /// List users.
    pub fn user_list(&self, list: &UserList) -> DriverResult<Vec<User>> {
        let conn = self.conn()?;
        ModelUser::list(&conn, list)
    }

    /// Create user.
    ///
    /// Returns error if email address is not unique.
    pub fn user_create(&self, create: &UserCreate) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::create(&conn, create)
    }

    /// Read user.
    pub fn user_read(&self, read: &UserRead) -> DriverResult<Option<User>> {
        let conn = self.conn()?;
        ModelUser::read(&conn, read)
    }

    /// Update user.
    pub fn user_update(&self, update: &UserUpdate) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::update(&conn, update)
    }

    /// Delete user.
    pub fn user_delete(&self, id: &Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelUser::delete(&conn, id)
    }
}
