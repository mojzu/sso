mod model;
mod schema;

use crate::{Audit, AuditCreate, Csrf, Driver, DriverError, Key, Service, User};
use chrono::{DateTime, Utc};
use diesel::{r2d2::ConnectionManager, PgConnection};
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
    pub fn initialise(
        database_url: &str,
        max_connections: Option<u32>,
    ) -> Result<Self, DriverError> {
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

    fn connection(&self) -> Result<PooledConnection, DriverError> {
        self.pool.get().map_err(DriverError::R2d2)
    }

    fn run_migrations(&self) -> Result<(), DriverError> {
        let connection = self.connection()?;
        embedded_migrations::run(&connection).map_err(DriverError::DieselMigrations)
    }
}

impl Driver for DriverPostgres {
    fn box_clone(&self) -> Box<dyn Driver> {
        Box::new((*self).clone())
    }

    fn audit_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Audit::list_where_id_lt(&conn, lt, limit, service_id_mask).map(Into::into)
    }

    fn audit_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Audit::list_where_id_gt(&conn, gt, limit, service_id_mask).map(Into::into)
    }

    fn audit_list_where_id_gt_and_lt(
        &self,
        gt: Uuid,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Audit::list_where_id_gt_and_lt(&conn, gt, lt, limit, service_id_mask).map(Into::into)
    }

    fn audit_list_where_created_lte(
        &self,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Audit::list_where_created_lte(&conn, created_lte, offset_id, limit, service_id_mask)
            .map(Into::into)
    }

    fn audit_list_where_created_gte(
        &self,
        created_gte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Audit::list_where_created_gte(&conn, created_gte, offset_id, limit, service_id_mask)
            .map(Into::into)
    }

    fn audit_list_where_created_gte_and_lte(
        &self,
        created_gte: &DateTime<Utc>,
        created_lte: &DateTime<Utc>,
        offset_id: Option<Uuid>,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Audit::list_where_created_gte_and_lte(
            &conn,
            created_gte,
            created_lte,
            offset_id,
            limit,
            service_id_mask,
        )
        .map(Into::into)
    }

    fn audit_create(&self, data: &AuditCreate) -> Result<Audit, DriverError> {
        let conn = self.connection()?;
        model::Audit::create(&conn, data).map(Into::into)
    }

    fn audit_read_by_id(
        &self,
        id: Uuid,
        service_id_mask: Option<Uuid>,
    ) -> Result<Option<Audit>, DriverError> {
        let conn = self.connection()?;
        model::Audit::read_by_id(&conn, id, service_id_mask).map(|x| x.map(|x| x.into()))
    }

    fn audit_read_metrics(
        &self,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<(String, i64)>, DriverError> {
        let conn = self.connection()?;
        model::Audit::read_metrics(&conn, service_id_mask)
    }

    fn audit_delete_by_created_at(
        &self,
        audit_created_at: &DateTime<Utc>,
    ) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Audit::delete_by_created_at(&conn, audit_created_at)
    }

    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        csrf_service_id: Uuid,
    ) -> Result<Csrf, DriverError> {
        let conn = self.connection()?;
        model::Csrf::create(&conn, key, value, ttl, csrf_service_id).map(Into::into)
    }

    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, DriverError> {
        let conn = self.connection()?;
        model::Csrf::read_by_key(&conn, key).map(|x| x.map(|x| x.into()))
    }

    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Csrf::delete_by_key(&conn, key)
    }

    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Csrf::delete_by_ttl(&conn, now)
    }

    fn key_list_where_id_lt(
        &self,
        lt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Key::list_where_id_lt(&conn, lt, limit, service_id_mask).map(Into::into)
    }

    fn key_list_where_id_gt(
        &self,
        gt: Uuid,
        limit: i64,
        service_id_mask: Option<Uuid>,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Key::list_where_id_gt(&conn, gt, limit, service_id_mask).map(Into::into)
    }

    fn key_create(
        &self,
        is_enabled: bool,
        is_revoked: bool,
        name: &str,
        value: &str,
        key_service_id: Option<Uuid>,
        key_user_id: Option<Uuid>,
    ) -> Result<Key, DriverError> {
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
        .map(Into::into)
    }

    fn key_read_by_id(&self, id: Uuid) -> Result<Option<Key>, DriverError> {
        let conn = self.connection()?;
        model::Key::read_by_id(&conn, id).map(|x| x.map(|x| x.into()))
    }

    fn key_read_by_user_id(
        &self,
        key_service_id: Uuid,
        key_user_id: Uuid,
    ) -> Result<Option<Key>, DriverError> {
        let conn = self.connection()?;
        model::Key::read_by_user_id(&conn, key_service_id, key_user_id).map(|x| x.map(|x| x.into()))
    }

    fn key_read_by_root_value(&self, value: &str) -> Result<Option<Key>, DriverError> {
        let conn = self.connection()?;
        model::Key::read_by_root_value(&conn, value).map(|x| x.map(|x| x.into()))
    }

    fn key_read_by_service_value(&self, value: &str) -> Result<Option<Key>, DriverError> {
        let conn = self.connection()?;
        model::Key::read_by_service_value(&conn, value).map(|x| x.map(|x| x.into()))
    }

    fn key_read_by_user_value(
        &self,
        key_service_id: Uuid,
        value: &str,
    ) -> Result<Option<Key>, DriverError> {
        let conn = self.connection()?;
        model::Key::read_by_user_value(&conn, key_service_id, value).map(|x| x.map(|x| x.into()))
    }

    fn key_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<Key, DriverError> {
        let conn = self.connection()?;
        model::Key::update_by_id(&conn, id, is_enabled, is_revoked, name).map(Into::into)
    }

    fn key_update_many_by_user_id(
        &self,
        key_user_id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<&str>,
    ) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Key::update_many_by_user_id(&conn, key_user_id, is_enabled, is_revoked, name)
    }

    fn key_delete_by_id(&self, id: Uuid) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Key::delete_by_id(&conn, id)
    }

    fn key_delete_root(&self) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Key::delete_root(&conn)
    }

    fn service_list_where_id_lt(&self, lt: Uuid, limit: i64) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Service::list_where_id_lt(&conn, lt, limit).map(Into::into)
    }

    fn service_list_where_id_gt(&self, gt: Uuid, limit: i64) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::Service::list_where_id_gt(&conn, gt, limit).map(Into::into)
    }

    fn service_create(
        &self,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> Result<Service, DriverError> {
        let conn = self.connection()?;
        model::Service::create(&conn, is_enabled, name, url).map(Into::into)
    }

    fn service_read_by_id(&self, id: Uuid) -> Result<Option<Service>, DriverError> {
        let conn = self.connection()?;
        model::Service::read_by_id(&conn, id).map(|x| x.map(|x| x.into()))
    }

    fn service_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<Service, DriverError> {
        let conn = self.connection()?;
        model::Service::update_by_id(&conn, id, is_enabled, name).map(Into::into)
    }

    fn service_delete_by_id(&self, id: Uuid) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::Service::delete_by_id(&conn, id)
    }

    fn user_list_where_id_lt(&self, lt: Uuid, limit: i64) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::User::list_where_id_lt(&conn, lt, limit)
    }

    fn user_list_where_id_gt(&self, gt: Uuid, limit: i64) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::User::list_where_id_gt(&conn, gt, limit)
    }

    fn user_list_where_email_eq(
        &self,
        email_eq: &str,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError> {
        let conn = self.connection()?;
        model::User::list_where_email_eq(&conn, email_eq, limit)
    }

    fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, DriverError> {
        let conn = self.connection()?;
        model::User::create(&conn, is_enabled, name, email, password_hash).map(Into::into)
    }

    fn user_read_by_id(&self, id: Uuid) -> Result<Option<User>, DriverError> {
        let conn = self.connection()?;
        model::User::read_by_id(&conn, id).map(|x| x.map(|x| x.into()))
    }

    fn user_read_by_email(&self, email: &str) -> Result<Option<User>, DriverError> {
        let conn = self.connection()?;
        model::User::read_by_email(&conn, email).map(|x| x.map(|x| x.into()))
    }

    fn user_update_by_id(
        &self,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<User, DriverError> {
        let conn = self.connection()?;
        model::User::update_by_id(&conn, id, is_enabled, name).map(Into::into)
    }

    fn user_update_email_by_id(&self, id: Uuid, email: &str) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::User::update_email_by_id(&conn, id, email)
    }

    fn user_update_password_by_id(
        &self,
        id: Uuid,
        password_hash: &str,
    ) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::User::update_password_by_id(&conn, id, password_hash)
    }

    fn user_delete_by_id(&self, id: Uuid) -> Result<usize, DriverError> {
        let conn = self.connection()?;
        model::User::delete_by_id(&conn, id)
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
