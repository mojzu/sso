//! # PostgreSQL Driver
mod models;
mod schema;

use crate::{core, driver};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

embed_migrations!("migrations/postgres");

#[derive(Clone)]
pub struct Driver {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

impl Driver {
    pub fn initialise(database_url: &str) -> Result<Self, driver::Error> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
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

    fn service_read_by_key_value(
        &self,
        key_value: &str,
    ) -> Result<Option<core::Service>, driver::Error> {
        unimplemented!();
    }

    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        csrf_service_id: i64,
    ) -> Result<core::Csrf, driver::Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        let value = models::AuthCsrfInsert {
            csrf_key: key,
            csrf_value: value,
            service_id: csrf_service_id,
        };
        diesel::insert_into(auth_csrf)
            .values(&value)
            .get_result::<models::AuthCsrf>(&conn)
            .map_err(driver::Error::Diesel)
            .map(Into::into)
    }

    fn csrf_read_by_key(&self, key: &str) -> Result<Option<core::Csrf>, driver::Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        auth_csrf
            .filter(csrf_key.eq(key))
            .get_result::<models::AuthCsrf>(&conn)
            .map(|csrf| Some(csrf.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(driver::Error::Diesel(err)),
            })
    }

    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, driver::Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_csrf.filter(csrf_key.eq(key)))
            .execute(&conn)
            .map_err(driver::Error::Diesel)
    }

    fn csrf_delete_by_created_at(
        &self,
        csrf_created_at: &DateTime<Utc>,
    ) -> Result<usize, driver::Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_csrf.filter(created_at.le(csrf_created_at)))
            .execute(&conn)
            .map_err(driver::Error::Diesel)
    }
}
