//! # SQLite Driver
mod model;
mod schema;

use crate::core::{Audit, Csrf, Key, Service, User};
use crate::driver;
use crate::driver::Error;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use serde_json::Value;

embed_migrations!("migrations/sqlite");

#[derive(Clone)]
pub struct Driver {
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

// TODO(feature): Implement SQLite driver.

impl Driver {
    pub fn initialise(database_url: &str, max_connections: u32) -> Result<Self, driver::Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .build(manager)
            .map_err(driver::Error::R2d2)?;
        let driver = Driver { pool };
        driver.run_migrations()?;
        Ok(driver)
    }

    fn connection(&self) -> Result<PooledConnection, driver::Error> {
        self.pool.get().map_err(driver::Error::R2d2)
    }

    fn run_migrations(&self) -> Result<(), driver::Error> {
        let connection = self.connection()?;
        embedded_migrations::run(&connection).map_err(driver::Error::DieselMigrations)
    }
}

impl driver::Driver for Driver {
    fn box_clone(&self) -> Box<driver::Driver> {
        Box::new((*self).clone())
    }

    fn key_list_where_id_lt(
        &self,
        lt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn key_list_where_id_gt(
        &self,
        gt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn key_create(
        &self,
        is_enabled: bool,
        name: &str,
        value: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
    ) -> Result<Key, Error> {
        unimplemented!();
    }

    fn key_read_by_id(&self, id: &str) -> Result<Option<Key>, Error> {
        unimplemented!();
    }

    fn key_read_by_user_id(&self, service_id: &str, user_id: &str) -> Result<Option<Key>, Error> {
        unimplemented!();
    }

    fn key_read_by_root_value(&self, value: &str) -> Result<Option<Key>, Error> {
        unimplemented!();
    }

    fn key_read_by_service_value(&self, value: &str) -> Result<Option<Key>, Error> {
        unimplemented!();
    }

    fn key_read_by_user_value(&self, service_id: &str, value: &str) -> Result<Option<Key>, Error> {
        unimplemented!();
    }

    fn key_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<Key, Error> {
        unimplemented!();
    }

    fn key_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        unimplemented!();
    }

    fn key_delete_root(&self) -> Result<usize, Error> {
        unimplemented!();
    }

    fn service_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn service_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn service_create(&self, is_enabled: bool, name: &str, url: &str) -> Result<Service, Error> {
        unimplemented!();
    }

    fn service_read_by_id(&self, id: &str) -> Result<Option<Service>, Error> {
        unimplemented!();
    }

    fn service_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<Service, Error> {
        unimplemented!();
    }

    fn service_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        unimplemented!();
    }

    fn user_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn user_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> Result<Vec<String>, Error> {
        unimplemented!();
    }

    fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, Error> {
        unimplemented!();
    }

    fn user_read_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        unimplemented!();
    }

    fn user_read_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        unimplemented!();
    }

    fn user_update_by_id(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<User, Error> {
        unimplemented!();
    }

    fn user_update_password_by_id(&self, id: &str, password_hash: &str) -> Result<usize, Error> {
        unimplemented!();
    }

    fn user_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        unimplemented!();
    }

    fn csrf_create(&self, key: &str, value: &str, service_id: &str) -> Result<Csrf, Error> {
        unimplemented!();
    }

    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, Error> {
        unimplemented!();
    }

    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, Error> {
        unimplemented!();
    }

    fn csrf_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> Result<usize, Error> {
        unimplemented!();
    }

    fn audit_create(
        &self,
        user_agent: &str,
        remote: &str,
        forwarded_for: Option<&str>,
        key: &str,
        data: &Value,
        key_id: &str,
        service_id: Option<&str>,
        user_id: Option<&str>,
        user_key_id: Option<&str>,
    ) -> Result<Audit, Error> {
        unimplemented!();
    }

    fn audit_delete_by_created_at(&self, created_at: &DateTime<Utc>) -> Result<usize, Error> {
        unimplemented!();
    }
}
