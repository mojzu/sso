mod diesel_admin;
mod model;
mod schema;

use crate::{
    driver::postgres::model::{ModelAudit, ModelCsrf, ModelKey, ModelService, ModelUser},
    Audit, AuditCreate, AuditList, Csrf, CsrfCreate, CsrfDelete, Driver, DriverError, DriverIf,
    DriverLock, DriverLockFn, DriverResult, Key, KeyCount, KeyCreate, KeyList, KeyRead, KeyUpdate,
    KeyWithValue, Service, ServiceCreate, ServiceList, ServiceUpdate, User, UserCreate, UserList,
    UserRead, UserUpdate, UserUpdate2,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, r2d2::ConnectionManager};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

embed_migrations!("migrations/postgres");

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Driver for PostgreSQL.
#[derive(Clone)]
pub struct DriverPostgres {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl fmt::Debug for DriverPostgres {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DriverPostgres {{ pool }}")
    }
}

impl DriverPostgres {
    /// Initialise driver with connection URL and number of pooled connections.
    pub fn initialise(database_url: &str, max_connections: Option<u32>) -> DriverResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let mut pool = r2d2::Pool::builder();
        if let Some(max_connections) = max_connections {
            pool = pool.max_size(max_connections);
        }
        let pool = pool.build(manager).map_err(DriverError::R2d2)?;
        let driver = DriverPostgres { pool };
        driver.run_migrations()?;
        Ok(driver)
    }

    fn conn(&self) -> DriverResult<PooledConnection> {
        self.pool.get().map_err(DriverError::R2d2)
    }

    fn run_migrations(&self) -> DriverResult<()> {
        let connection = self.conn()?;
        embedded_migrations::run(&connection).map_err(DriverError::DieselMigrations)
    }
}

impl DriverIf for DriverPostgres {
    fn exclusive_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<bool> {
        use diesel_admin::*;

        let conn = self.conn()?;
        conn.transaction(|| {
            if diesel::select(pg_try_advisory_xact_lock(1, key as i32)).get_result::<bool>(&conn)? {
                let conn_driver = DriverPostgresConnRef::new(&conn);
                func(&conn_driver)
            } else {
                Err(DriverError::Locked(key))
            }
        })
    }

    fn shared_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<bool> {
        use diesel_admin::*;

        let conn = self.conn()?;
        conn.transaction(|| {
            if diesel::select(pg_try_advisory_xact_lock_shared(1, key as i32))
                .get_result::<bool>(&conn)?
            {
                let conn_driver = DriverPostgresConnRef::new(&conn);
                func(&conn_driver)
            } else {
                Err(DriverError::Locked(key))
            }
        })
    }

    fn audit_list(&self, list: &AuditList) -> DriverResult<Vec<Audit>> {
        let conn = self.conn()?;
        ModelAudit::list(&conn, list)
    }

    fn audit_create(&self, create: &AuditCreate) -> DriverResult<Audit> {
        let conn = self.conn()?;
        ModelAudit::create(&conn, create)
    }

    fn audit_read_opt(
        &self,
        id: &Uuid,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Option<Audit>> {
        let conn = self.conn()?;
        ModelAudit::read_opt(&conn, id, service_id_mask)
    }

    fn audit_read_metrics(
        &self,
        from: &DateTime<Utc>,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
        let conn = self.conn()?;
        ModelAudit::read_metrics(&conn, from, service_id_mask)
    }

    fn audit_update(
        &self,
        id: &Uuid,
        data: &Value,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Audit> {
        let conn = self.conn()?;
        ModelAudit::update(&conn, id, data, service_id_mask)
    }

    fn audit_delete(&self, created_at: &DateTime<Utc>) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelAudit::delete(&conn, created_at)
    }

    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf> {
        let conn = self.conn()?;
        ModelCsrf::create(&conn, create)
    }

    fn csrf_read_opt(&self, key: &str) -> DriverResult<Option<Csrf>> {
        let conn = self.conn()?;
        ModelCsrf::read_opt(&conn, key)
    }

    fn csrf_delete(&self, delete: &CsrfDelete) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelCsrf::delete(&conn, delete)
    }

    fn key_list(&self, list: &KeyList) -> DriverResult<Vec<Key>> {
        let conn = self.conn()?;
        ModelKey::list(&conn, list)
    }

    fn key_count(&self, count: &KeyCount) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::count(&conn, count)
    }

    fn key_create(&self, create: &KeyCreate) -> DriverResult<KeyWithValue> {
        let conn = self.conn()?;
        ModelKey::create(&conn, create)
    }

    fn key_read_opt(&self, read: &KeyRead) -> DriverResult<Option<KeyWithValue>> {
        let conn = self.conn()?;
        ModelKey::read_opt(&conn, read)
    }

    fn key_update(&self, id: &Uuid, update: &KeyUpdate) -> DriverResult<Key> {
        let conn = self.conn()?;
        ModelKey::update(&conn, id, update)
    }

    fn key_update_many(&self, user_id: &Uuid, update: &KeyUpdate) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::update_many(&conn, user_id, update)
    }

    fn key_delete(&self, id: &Uuid) -> DriverResult<()> {
        let conn = self.conn()?;
        ModelKey::delete(&conn, id)
    }

    fn service_list(&self, list: &ServiceList) -> DriverResult<Vec<Service>> {
        let conn = self.conn()?;
        ModelService::list(&conn, list)
    }

    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service> {
        let conn = self.conn()?;
        ModelService::create(&conn, create)
    }

    fn service_read_opt(&self, id: &Uuid) -> DriverResult<Option<Service>> {
        let conn = self.conn()?;
        ModelService::read_opt(&conn, id)
    }

    fn service_update(&self, id: &Uuid, update: &ServiceUpdate) -> DriverResult<Service> {
        let conn = self.conn()?;
        ModelService::update(&conn, id, update)
    }

    fn service_delete(&self, id: &Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelService::delete(&conn, id)
    }

    fn user_list(&self, list: &UserList) -> DriverResult<Vec<User>> {
        let conn = self.conn()?;
        ModelUser::list(&conn, list)
    }

    fn user_create(&self, create: &UserCreate) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::create(&conn, create)
    }

    fn user_read(&self, read: &UserRead) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::read(&conn, read)
    }

    fn user_read_opt(&self, read: &UserRead) -> DriverResult<Option<User>> {
        let conn = self.conn()?;
        ModelUser::read_opt(&conn, read)
    }

    fn user_update(&self, id: &Uuid, update: &UserUpdate) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::update(&conn, id, update)
    }

    fn user_update2(&self, id: &Uuid, update: &UserUpdate2) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::update2(&conn, id, update)
    }

    fn user_delete(&self, id: &Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelUser::delete(&conn, id)
    }
}

impl Driver for DriverPostgres {
    fn box_clone(&self) -> Box<dyn Driver> {
        Box::new((*self).clone())
    }

    fn as_if(&self) -> &dyn DriverIf {
        self
    }
}

/// Driver for PostgreSQL connection reference.
struct DriverPostgresConnRef<'a> {
    conn: &'a PgConnection,
}

impl<'a> DriverPostgresConnRef<'a> {
    fn new(conn: &'a PgConnection) -> Self {
        Self { conn }
    }

    fn conn(&self) -> &'a PgConnection {
        self.conn
    }
}

impl<'a> DriverIf for DriverPostgresConnRef<'a> {
    fn exclusive_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<bool> {
        use diesel_admin::*;

        let conn = self.conn();
        conn.transaction(|| {
            if diesel::select(pg_try_advisory_xact_lock(1, key as i32)).get_result::<bool>(conn)? {
                let conn_driver = DriverPostgresConnRef::new(conn);
                func(&conn_driver)
            } else {
                Err(DriverError::Locked(key))
            }
        })
    }

    fn shared_lock(&self, key: DriverLock, func: DriverLockFn) -> DriverResult<bool> {
        use diesel_admin::*;

        let conn = self.conn();
        conn.transaction(|| {
            if diesel::select(pg_try_advisory_xact_lock_shared(1, key as i32))
                .get_result::<bool>(conn)?
            {
                let conn_driver = DriverPostgresConnRef::new(conn);
                func(&conn_driver)
            } else {
                Err(DriverError::Locked(key))
            }
        })
    }

    fn audit_list(&self, list: &AuditList) -> DriverResult<Vec<Audit>> {
        ModelAudit::list(self.conn(), list)
    }

    fn audit_create(&self, create: &AuditCreate) -> DriverResult<Audit> {
        ModelAudit::create(self.conn(), create)
    }

    fn audit_read_opt(
        &self,
        id: &Uuid,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Option<Audit>> {
        ModelAudit::read_opt(self.conn(), id, service_id_mask)
    }

    fn audit_read_metrics(
        &self,
        from: &DateTime<Utc>,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
        ModelAudit::read_metrics(self.conn(), from, service_id_mask)
    }

    fn audit_update(
        &self,
        id: &Uuid,
        data: &Value,
        service_id_mask: Option<&Uuid>,
    ) -> DriverResult<Audit> {
        ModelAudit::update(self.conn(), id, data, service_id_mask)
    }

    fn audit_delete(&self, created_at: &DateTime<Utc>) -> DriverResult<usize> {
        ModelAudit::delete(self.conn(), created_at)
    }

    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf> {
        ModelCsrf::create(self.conn(), create)
    }

    fn csrf_read_opt(&self, key: &str) -> DriverResult<Option<Csrf>> {
        ModelCsrf::read_opt(self.conn(), key)
    }

    fn csrf_delete(&self, delete: &CsrfDelete) -> DriverResult<usize> {
        ModelCsrf::delete(self.conn(), delete)
    }

    fn key_list(&self, list: &KeyList) -> DriverResult<Vec<Key>> {
        ModelKey::list(self.conn(), list)
    }

    fn key_count(&self, count: &KeyCount) -> DriverResult<usize> {
        ModelKey::count(self.conn(), count)
    }

    fn key_create(&self, create: &KeyCreate) -> DriverResult<KeyWithValue> {
        ModelKey::create(self.conn(), create)
    }

    fn key_read_opt(&self, read: &KeyRead) -> DriverResult<Option<KeyWithValue>> {
        ModelKey::read_opt(self.conn(), read)
    }

    fn key_update(&self, id: &Uuid, update: &KeyUpdate) -> DriverResult<Key> {
        ModelKey::update(self.conn(), id, update)
    }

    fn key_update_many(&self, user_id: &Uuid, update: &KeyUpdate) -> DriverResult<usize> {
        ModelKey::update_many(self.conn(), user_id, update)
    }

    fn key_delete(&self, id: &Uuid) -> DriverResult<()> {
        ModelKey::delete(self.conn(), id)
    }

    fn service_list(&self, list: &ServiceList) -> DriverResult<Vec<Service>> {
        ModelService::list(self.conn(), list)
    }

    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service> {
        ModelService::create(self.conn(), create)
    }

    fn service_read_opt(&self, id: &Uuid) -> DriverResult<Option<Service>> {
        ModelService::read_opt(self.conn(), id)
    }

    fn service_update(&self, id: &Uuid, update: &ServiceUpdate) -> DriverResult<Service> {
        ModelService::update(self.conn(), id, update)
    }

    fn service_delete(&self, id: &Uuid) -> DriverResult<usize> {
        ModelService::delete(self.conn(), id)
    }

    fn user_list(&self, list: &UserList) -> DriverResult<Vec<User>> {
        ModelUser::list(self.conn(), list)
    }

    fn user_create(&self, create: &UserCreate) -> DriverResult<User> {
        ModelUser::create(self.conn(), create)
    }

    fn user_read(&self, read: &UserRead) -> DriverResult<User> {
        ModelUser::read(self.conn(), read)
    }

    fn user_read_opt(&self, read: &UserRead) -> DriverResult<Option<User>> {
        ModelUser::read_opt(self.conn(), read)
    }

    fn user_update(&self, id: &Uuid, update: &UserUpdate) -> DriverResult<User> {
        ModelUser::update(self.conn(), id, update)
    }

    fn user_update2(&self, id: &Uuid, update: &UserUpdate2) -> DriverResult<User> {
        ModelUser::update2(self.conn(), id, update)
    }

    fn user_delete(&self, id: &Uuid) -> DriverResult<usize> {
        ModelUser::delete(self.conn(), id)
    }
}
