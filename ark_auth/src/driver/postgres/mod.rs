mod diesel_admin;
mod model;
mod schema;

use crate::{
    driver::postgres::model::{ModelAudit, ModelCsrf, ModelKey, ModelService, ModelUser},
    Audit, AuditCreate, Csrf, CsrfCreate, Driver, DriverError, DriverIf, DriverLockFn,
    DriverResult, Key, KeyCreate, KeyUpdate, Service, ServiceCreate, ServiceUpdate, User,
    UserCreate, UserUpdate,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, r2d2::ConnectionManager};
use uuid::Uuid;

embed_migrations!("migrations/postgres");

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Driver for PostgreSQL.
#[derive(Clone)]
pub struct DriverPostgres {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
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
    fn exclusive_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()> {
        use diesel_admin::*;

        let conn = self.conn()?;
        conn.transaction(|| {
            diesel::select(pg_advisory_xact_lock(1, key)).execute(&conn)?;
            let conn_driver = DriverPostgresConn::new(&conn);
            func(&conn_driver);
            Ok(())
        })
        .map_err(DriverError::DieselResult)
    }

    fn shared_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()> {
        use diesel_admin::*;

        let conn = self.conn()?;
        conn.transaction(|| {
            diesel::select(pg_advisory_xact_lock_shared(1, key)).execute(&conn)?;
            let conn_driver = DriverPostgresConn::new(&conn);
            func(&conn_driver);
            Ok(())
        })
        .map_err(DriverError::DieselResult)
    }

    fn audit_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelAudit::list_where_id_lt(&conn, lt, limit, service_id_mask)
    }

    fn audit_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelAudit::list_where_id_gt(&conn, gt, limit, service_id_mask)
    }

    fn audit_list_where_id_gt_and_lt(
        &self,
        gt: Uuid,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelAudit::list_where_id_gt_and_lt(&conn, gt, lt, limit, service_id_mask)
    }

    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelAudit::list_where_created_lte(&conn, created_lte, offset_id, limit, service_id_mask)
    }

    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelAudit::list_where_created_gte(&conn, created_gte, offset_id, limit, service_id_mask)
    }

    fn audit_list_where_created_gte_and_lte(
        &self,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelAudit::list_where_created_gte_and_lte(
            &conn,
            created_gte,
            created_lte,
            offset_id,
            limit,
            service_id_mask,
        )
    }

    fn audit_create(&self, create: &AuditCreate) -> DriverResult<Audit> {
        let conn = self.conn()?;
        ModelAudit::create(&conn, create)
    }

    fn audit_read_by_id(
        &self,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Option<Audit>> {
        let conn = self.conn()?;
        ModelAudit::read_by_id(&conn, id, service_id_mask)
    }

    fn audit_read_metrics(
        &self,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
        let conn = self.conn()?;
        ModelAudit::read_metrics(&conn, service_id_mask)
    }

    fn audit_delete_by_created_at(&self, audit_created_at: &DateTime<Utc>) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelAudit::delete_by_created_at(&conn, audit_created_at)
    }

    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf> {
        let conn = self.conn()?;
        ModelCsrf::create(&conn, create)
    }

    fn csrf_read_by_key(&self, key: &str) -> DriverResult<Option<Csrf>> {
        let conn = self.conn()?;
        ModelCsrf::read_by_key(&conn, key)
    }

    fn csrf_delete_by_key(&self, key: &str) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelCsrf::delete_by_key(&conn, key)
    }

    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelCsrf::delete_by_ttl(&conn, now)
    }

    fn key_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelKey::list_where_id_lt(&conn, lt, limit, service_id_mask)
    }

    fn key_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelKey::list_where_id_gt(&conn, gt, limit, service_id_mask)
    }

    fn key_create(&self, create: &KeyCreate) -> DriverResult<Key> {
        let conn = self.conn()?;
        ModelKey::create(&conn, create)
    }

    fn key_read_by_id(&self, id: Uuid) -> DriverResult<Option<Key>> {
        let conn = self.conn()?;
        ModelKey::read_by_id(&conn, id)
    }

    fn key_read_by_user_id(
        &self,
        service_id: Uuid,
        user_id: Uuid,
        is_enabled: bool,
        is_revoked: bool,
    ) -> DriverResult<Option<Key>> {
        let conn = self.conn()?;
        ModelKey::read_by_user_id(&conn, service_id, user_id, is_enabled, is_revoked)
    }

    fn key_read_by_root_value(&self, value: &str) -> DriverResult<Option<Key>> {
        let conn = self.conn()?;
        ModelKey::read_by_root_value(&conn, value)
    }

    fn key_read_by_service_value(&self, value: &str) -> DriverResult<Option<Key>> {
        let conn = self.conn()?;
        ModelKey::read_by_service_value(&conn, value)
    }

    fn key_read_by_user_value(
        &self,
        key_service_id: Uuid,
        value: &str,
    ) -> DriverResult<Option<Key>> {
        let conn = self.conn()?;
        ModelKey::read_by_user_value(&conn, key_service_id, value)
    }

    fn key_update_by_id(&self, id: Uuid, update: &KeyUpdate) -> DriverResult<Key> {
        let conn = self.conn()?;
        ModelKey::update_by_id(&conn, id, update)
    }

    fn key_update_many_by_user_id(&self, user_id: Uuid, update: &KeyUpdate) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::update_many_by_user_id(&conn, user_id, update)
    }

    fn key_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::delete_by_id(&conn, id)
    }

    fn key_delete_root(&self) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelKey::delete_root(&conn)
    }

    fn service_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelService::list_where_id_lt(&conn, lt, limit)
    }

    fn service_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelService::list_where_id_gt(&conn, gt, limit)
    }

    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service> {
        let conn = self.conn()?;
        ModelService::create(&conn, create)
    }

    fn service_read_by_id(&self, id: Uuid) -> DriverResult<Option<Service>> {
        let conn = self.conn()?;
        ModelService::read_by_id(&conn, id)
    }

    fn service_update_by_id(&self, id: Uuid, update: &ServiceUpdate) -> DriverResult<Service> {
        let conn = self.conn()?;
        ModelService::update_by_id(&conn, id, update)
    }

    fn service_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelService::delete_by_id(&conn, id)
    }

    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelUser::list_where_id_lt(&conn, lt, limit)
    }

    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelUser::list_where_id_gt(&conn, gt, limit)
    }

    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.conn()?;
        ModelUser::list_where_email_eq(&conn, email_eq, limit)
    }

    fn user_create(&self, create: &UserCreate) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::create(&conn, create)
    }

    fn user_read_by_id(&self, id: Uuid) -> DriverResult<Option<User>> {
        let conn = self.conn()?;
        ModelUser::read_by_id(&conn, id)
    }

    fn user_read_by_email(&self, email: &str) -> DriverResult<Option<User>> {
        let conn = self.conn()?;
        ModelUser::read_by_email(&conn, email)
    }

    fn user_update_by_id(&self, id: Uuid, update: &UserUpdate) -> DriverResult<User> {
        let conn = self.conn()?;
        ModelUser::update_by_id(&conn, id, update)
    }

    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelUser::update_email_by_id(&conn, id, email)
    }

    fn user_update_password_by_id(&self, id: Uuid, password_hash: &str) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelUser::update_password_by_id(&conn, id, password_hash)
    }

    fn user_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        let conn = self.conn()?;
        ModelUser::delete_by_id(&conn, id)
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
struct DriverPostgresConn<'a> {
    conn: &'a PgConnection,
}

impl<'a> DriverPostgresConn<'a> {
    fn new(conn: &'a PgConnection) -> Self {
        Self { conn }
    }

    fn conn(&self) -> &'a PgConnection {
        self.conn
    }
}

impl<'a> DriverIf for DriverPostgresConn<'a> {
    fn exclusive_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()> {
        use diesel_admin::*;

        let conn = self.conn();
        conn.transaction(|| {
            diesel::select(pg_advisory_xact_lock(1, key)).execute(conn)?;
            let conn_driver = DriverPostgresConn::new(conn);
            func(&conn_driver);
            Ok(())
        })
        .map_err(DriverError::DieselResult)
    }

    fn shared_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()> {
        use diesel_admin::*;

        let conn = self.conn();
        conn.transaction(|| {
            diesel::select(pg_advisory_xact_lock_shared(1, key)).execute(conn)?;
            let conn_driver = DriverPostgresConn::new(conn);
            func(&conn_driver);
            Ok(())
        })
        .map_err(DriverError::DieselResult)
    }

    fn audit_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelAudit::list_where_id_lt(self.conn(), lt, limit, service_id_mask)
    }

    fn audit_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelAudit::list_where_id_gt(self.conn(), gt, limit, service_id_mask)
    }

    fn audit_list_where_id_gt_and_lt(
        &self,
        gt: Uuid,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelAudit::list_where_id_gt_and_lt(self.conn(), gt, lt, limit, service_id_mask)
    }

    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelAudit::list_where_created_lte(
            self.conn(),
            created_lte,
            offset_id,
            limit,
            service_id_mask,
        )
    }

    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelAudit::list_where_created_gte(
            self.conn(),
            created_gte,
            offset_id,
            limit,
            service_id_mask,
        )
    }

    fn audit_list_where_created_gte_and_lte(
        &self,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelAudit::list_where_created_gte_and_lte(
            self.conn(),
            created_gte,
            created_lte,
            offset_id,
            limit,
            service_id_mask,
        )
    }

    fn audit_create(&self, create: &AuditCreate) -> DriverResult<Audit> {
        ModelAudit::create(self.conn(), create)
    }

    fn audit_read_by_id(
        &self,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Option<Audit>> {
        ModelAudit::read_by_id(self.conn(), id, service_id_mask)
    }

    fn audit_read_metrics(
        &self,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
        ModelAudit::read_metrics(self.conn(), service_id_mask)
    }

    fn audit_delete_by_created_at(&self, audit_created_at: &DateTime<Utc>) -> DriverResult<usize> {
        ModelAudit::delete_by_created_at(self.conn(), audit_created_at)
    }

    fn csrf_create(&self, create: &CsrfCreate) -> DriverResult<Csrf> {
        ModelCsrf::create(self.conn(), create)
    }

    fn csrf_read_by_key(&self, key: &str) -> DriverResult<Option<Csrf>> {
        ModelCsrf::read_by_key(self.conn(), key)
    }

    fn csrf_delete_by_key(&self, key: &str) -> DriverResult<usize> {
        ModelCsrf::delete_by_key(self.conn(), key)
    }

    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> DriverResult<usize> {
        ModelCsrf::delete_by_ttl(self.conn(), now)
    }

    fn key_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelKey::list_where_id_lt(self.conn(), lt, limit, service_id_mask)
    }

    fn key_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        ModelKey::list_where_id_gt(self.conn(), gt, limit, service_id_mask)
    }

    fn key_create(&self, create: &KeyCreate) -> DriverResult<Key> {
        ModelKey::create(self.conn(), create)
    }

    fn key_read_by_id(&self, id: Uuid) -> DriverResult<Option<Key>> {
        ModelKey::read_by_id(self.conn(), id)
    }

    fn key_read_by_user_id(
        &self,
        service_id: Uuid,
        user_id: Uuid,
        is_enabled: bool,
        is_revoked: bool,
    ) -> DriverResult<Option<Key>> {
        ModelKey::read_by_user_id(self.conn(), service_id, user_id, is_enabled, is_revoked)
    }

    fn key_read_by_root_value(&self, value: &str) -> DriverResult<Option<Key>> {
        ModelKey::read_by_root_value(self.conn(), value)
    }

    fn key_read_by_service_value(&self, value: &str) -> DriverResult<Option<Key>> {
        ModelKey::read_by_service_value(self.conn(), value)
    }

    fn key_read_by_user_value(
        &self,
        key_service_id: Uuid,
        value: &str,
    ) -> DriverResult<Option<Key>> {
        ModelKey::read_by_user_value(self.conn(), key_service_id, value)
    }

    fn key_update_by_id(&self, id: Uuid, update: &KeyUpdate) -> DriverResult<Key> {
        ModelKey::update_by_id(self.conn(), id, update)
    }

    fn key_update_many_by_user_id(&self, user_id: Uuid, update: &KeyUpdate) -> DriverResult<usize> {
        ModelKey::update_many_by_user_id(self.conn(), user_id, update)
    }

    fn key_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        ModelKey::delete_by_id(self.conn(), id)
    }

    fn key_delete_root(&self) -> DriverResult<usize> {
        ModelKey::delete_root(self.conn())
    }

    fn service_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        ModelService::list_where_id_lt(self.conn(), lt, limit)
    }

    fn service_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        ModelService::list_where_id_gt(self.conn(), gt, limit)
    }

    fn service_create(&self, create: &ServiceCreate) -> DriverResult<Service> {
        ModelService::create(self.conn(), create)
    }

    fn service_read_by_id(&self, id: Uuid) -> DriverResult<Option<Service>> {
        ModelService::read_by_id(self.conn(), id)
    }

    fn service_update_by_id(&self, id: Uuid, update: &ServiceUpdate) -> DriverResult<Service> {
        ModelService::update_by_id(self.conn(), id, update)
    }

    fn service_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        ModelService::delete_by_id(self.conn(), id)
    }

    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        ModelUser::list_where_id_lt(self.conn(), lt, limit)
    }

    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        ModelUser::list_where_id_gt(self.conn(), gt, limit)
    }

    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> DriverResult<Vec<Uuid>> {
        ModelUser::list_where_email_eq(self.conn(), email_eq, limit)
    }

    fn user_create(&self, create: &UserCreate) -> DriverResult<User> {
        ModelUser::create(self.conn(), create)
    }

    fn user_read_by_id(&self, id: Uuid) -> DriverResult<Option<User>> {
        ModelUser::read_by_id(self.conn(), id)
    }

    fn user_read_by_email(&self, email: &str) -> DriverResult<Option<User>> {
        ModelUser::read_by_email(self.conn(), email)
    }

    fn user_update_by_id(&self, id: Uuid, update: &UserUpdate) -> DriverResult<User> {
        ModelUser::update_by_id(self.conn(), id, update)
    }

    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> DriverResult<usize> {
        ModelUser::update_email_by_id(self.conn(), id, email)
    }

    fn user_update_password_by_id(&self, id: Uuid, password_hash: &str) -> DriverResult<usize> {
        ModelUser::update_password_by_id(self.conn(), id, password_hash)
    }

    fn user_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        ModelUser::delete_by_id(self.conn(), id)
    }
}
