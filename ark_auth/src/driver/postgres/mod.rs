mod diesel_admin;
mod model;
mod schema;

use crate::{
    Audit, AuditCreate, Csrf, Driver, DriverError, DriverLockFn, DriverResult, Key, Service, User,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, r2d2::ConnectionManager, PgConnection};
use uuid::Uuid;

embed_migrations!("migrations/postgres");

/// Driver for PostgreSQL.
#[derive(Clone)]
pub struct DriverPostgres {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

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

    fn connection(&self) -> DriverResult<PooledConnection> {
        self.pool.get().map_err(DriverError::R2d2)
    }

    fn run_migrations(&self) -> DriverResult<()> {
        let connection = self.connection()?;
        embedded_migrations::run(&connection).map_err(DriverError::DieselMigrations)
    }
}

impl Driver for DriverPostgres {
    fn box_clone(&self) -> Box<dyn Driver> {
        Box::new((*self).clone())
    }

    fn exclusive_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()> {
        use diesel_admin::*;

        let conn_driver = self.box_clone();
        let conn = self.connection()?;
        conn.transaction(|| {
            diesel::select(pg_advisory_xact_lock(1, key)).execute(&conn)?;
            func(conn_driver.as_ref());
            Ok(())
        })
        .map_err(DriverError::DieselResult)
    }

    fn shared_lock(&self, key: i32, func: DriverLockFn) -> DriverResult<()> {
        use diesel_admin::*;

        let conn_driver = self.box_clone();
        let conn = self.connection()?;
        conn.transaction(|| {
            diesel::select(pg_advisory_xact_lock_shared(1, key)).execute(&conn)?;
            func(conn_driver.as_ref());
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
        let conn = self.connection()?;
        model::Audit::list_where_id_lt(&conn, lt, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn audit_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Audit::list_where_id_gt(&conn, gt, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn audit_list_where_id_gt_and_lt(
        &self,
        gt: Uuid,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Audit::list_where_id_gt_and_lt(&conn, gt, lt, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Audit::list_where_created_lte(&conn, created_lte, offset_id, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Audit::list_where_created_gte(&conn, created_gte, offset_id, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn audit_list_where_created_gte_and_lte(
        &self,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Audit::list_where_created_gte_and_lte(
            &conn,
            created_gte,
            created_lte,
            offset_id,
            limit,
            service_id_mask,
        )
        .map_err(Into::into)
        .map(Into::into)
    }

    fn audit_create(&self, data: &AuditCreate) -> DriverResult<Audit> {
        let conn = self.connection()?;
        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = model::AuditInsert::from_create(&now, id, data);
        model::Audit::create(&conn, &value)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn audit_read_by_id(
        &self,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Option<Audit>> {
        let conn = self.connection()?;
        model::Audit::read_by_id(&conn, id, service_id_mask)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn audit_read_metrics(
        &self,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<(String, i64)>> {
        let conn = self.connection()?;
        model::Audit::read_metrics(&conn, service_id_mask).map_err(Into::into)
    }

    fn audit_delete_by_created_at(&self, audit_created_at: &DateTime<Utc>) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Audit::delete_by_created_at(&conn, audit_created_at).map_err(Into::into)
    }

    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        service_id: Uuid,
    ) -> DriverResult<Csrf> {
        let conn = self.connection()?;
        let now = Utc::now();
        let value = model::CsrfInsert::from(&now, key, value, ttl, service_id);
        model::Csrf::create(&conn, &value)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn csrf_read_by_key(&self, key: &str) -> DriverResult<Option<Csrf>> {
        let conn = self.connection()?;
        model::Csrf::read_by_key(&conn, key)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn csrf_delete_by_key(&self, key: &str) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Csrf::delete_by_key(&conn, key).map_err(Into::into)
    }

    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Csrf::delete_by_ttl(&conn, now).map_err(Into::into)
    }

    fn key_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Key::list_where_id_lt(&conn, lt, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn key_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Key::list_where_id_gt(&conn, gt, limit, service_id_mask)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn key_create(
        &self,
        is_enabled: bool,
        is_revoked: bool,
        name: &str,
        value: &str,
        key_service_id: Option<Uuid>,
        key_user_id: Option<Uuid>,
    ) -> DriverResult<Key> {
        let conn = self.connection()?;
        model::Key::create(
            &conn,
            is_enabled,
            is_revoked,
            name,
            value,
            key_service_id,
            key_user_id,
        )
        .map_err(Into::into)
        .map(Into::into)
    }

    fn key_read_by_id(&self, id: Uuid) -> DriverResult<Option<Key>> {
        let conn = self.connection()?;
        model::Key::read_by_id(&conn, id)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn key_read_by_user_id(
        &self,
        service_id: Uuid,
        user_id: Uuid,
        is_enabled: bool,
        is_revoked: bool,
    ) -> DriverResult<Option<Key>> {
        let conn = self.connection()?;
        model::Key::read_by_user_id(&conn, service_id, user_id, is_enabled, is_revoked)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn key_read_by_root_value(&self, value: &str) -> DriverResult<Option<Key>> {
        let conn = self.connection()?;
        model::Key::read_by_root_value(&conn, value)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn key_read_by_service_value(&self, value: &str) -> DriverResult<Option<Key>> {
        let conn = self.connection()?;
        model::Key::read_by_service_value(&conn, value)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn key_read_by_user_value(
        &self,
        key_service_id: Uuid,
        value: &str,
    ) -> DriverResult<Option<Key>> {
        let conn = self.connection()?;
        model::Key::read_by_user_value(&conn, key_service_id, value)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn key_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<Key> {
        let conn = self.connection()?;
        model::Key::update_by_id(&conn, id, is_enabled, is_revoked, name)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn key_update_many_by_user_id(
        &self,
        key_user_id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Key::update_many_by_user_id(&conn, key_user_id, is_enabled, is_revoked, name)
            .map_err(Into::into)
    }

    fn key_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Key::delete_by_id(&conn, id).map_err(Into::into)
    }

    fn key_delete_root(&self) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Key::delete_root(&conn).map_err(Into::into)
    }

    fn service_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Service::list_where_id_lt(&conn, lt, limit)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn service_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::Service::list_where_id_gt(&conn, gt, limit)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn service_create(&self, is_enabled: bool, name: &str, url: &str) -> DriverResult<Service> {
        let conn = self.connection()?;
        model::Service::create(&conn, is_enabled, name, url)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn service_read_by_id(&self, id: Uuid) -> DriverResult<Option<Service>> {
        let conn = self.connection()?;
        model::Service::read_by_id(&conn, id)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn service_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<Service> {
        let conn = self.connection()?;
        model::Service::update_by_id(&conn, id, is_enabled, name)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn service_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::Service::delete_by_id(&conn, id).map_err(Into::into)
    }

    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::User::list_where_id_lt(&conn, lt, limit).map_err(Into::into)
    }

    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::User::list_where_id_gt(&conn, gt, limit).map_err(Into::into)
    }

    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> DriverResult<Vec<Uuid>> {
        let conn = self.connection()?;
        model::User::list_where_email_eq(&conn, email_eq, limit).map_err(Into::into)
    }

    fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> DriverResult<User> {
        let conn = self.connection()?;
        model::User::create(&conn, is_enabled, name, email, password_hash)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn user_read_by_id(&self, id: Uuid) -> DriverResult<Option<User>> {
        let conn = self.connection()?;
        model::User::read_by_id(&conn, id)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn user_read_by_email(&self, email: &str) -> DriverResult<Option<User>> {
        let conn = self.connection()?;
        model::User::read_by_email(&conn, email)
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    fn user_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> DriverResult<User> {
        let conn = self.connection()?;
        model::User::update_by_id(&conn, id, is_enabled, name)
            .map_err(Into::into)
            .map(Into::into)
    }

    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::User::update_email_by_id(&conn, id, email).map_err(Into::into)
    }

    fn user_update_password_by_id(&self, id: Uuid, password_hash: &str) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::User::update_password_by_id(&conn, id, password_hash).map_err(Into::into)
    }

    fn user_delete_by_id(&self, id: Uuid) -> DriverResult<usize> {
        let conn = self.connection()?;
        model::User::delete_by_id(&conn, id).map_err(Into::into)
    }
}

impl From<model::Audit> for Audit {
    fn from(audit: model::Audit) -> Self {
        Audit {
            created_at: audit.created_at,
            id: audit.audit_id,
            user_agent: audit.audit_user_agent,
            remote: audit.audit_remote,
            forwarded: audit.audit_forwarded,
            path: audit.audit_path,
            data: audit.audit_data,
            key_id: audit.key_id,
            service_id: audit.service_id,
            user_id: audit.user_id,
            user_key_id: audit.user_key_id,
        }
    }
}

impl<'a> model::AuditInsert<'a> {
    pub fn from_create(now: &'a DateTime<Utc>, id: Uuid, create: &'a AuditCreate) -> Self {
        Self {
            created_at: now,
            audit_id: id,
            audit_user_agent: create.meta.user_agent(),
            audit_remote: create.meta.remote(),
            audit_forwarded: create.meta.forwarded(),
            audit_path: create.path,
            audit_data: create.data,
            key_id: create.key_id,
            service_id: create.service_id,
            user_id: create.user_id,
            user_key_id: create.user_key_id,
        }
    }
}

impl From<model::Csrf> for Csrf {
    fn from(csrf: model::Csrf) -> Self {
        Csrf {
            created_at: csrf.created_at,
            key: csrf.csrf_key,
            value: csrf.csrf_value,
            ttl: csrf.csrf_ttl,
            service_id: csrf.service_id,
        }
    }
}

impl<'a> model::CsrfInsert<'a> {
    pub fn from(
        now: &'a DateTime<Utc>,
        key: &'a str,
        value: &'a str,
        ttl: &'a DateTime<Utc>,
        service_id: Uuid,
    ) -> Self {
        Self {
            created_at: now,
            csrf_key: key,
            csrf_value: value,
            csrf_ttl: ttl,
            service_id,
        }
    }
}

impl From<model::Key> for Key {
    fn from(key: model::Key) -> Self {
        Key {
            created_at: key.created_at,
            updated_at: key.updated_at,
            id: key.key_id,
            is_enabled: key.key_is_enabled,
            is_revoked: key.key_is_revoked,
            name: key.key_name,
            value: key.key_value,
            service_id: key.service_id,
            user_id: key.user_id,
        }
    }
}

impl From<model::Service> for Service {
    fn from(service: model::Service) -> Self {
        Service {
            created_at: service.created_at,
            updated_at: service.updated_at,
            id: service.service_id,
            is_enabled: service.service_is_enabled,
            name: service.service_name,
            url: service.service_url,
        }
    }
}

impl From<model::User> for User {
    fn from(user: model::User) -> Self {
        User {
            created_at: user.created_at,
            updated_at: user.updated_at,
            id: user.user_id,
            is_enabled: user.user_is_enabled,
            name: user.user_name,
            email: user.user_email,
            password_hash: user.user_password_hash,
        }
    }
}
