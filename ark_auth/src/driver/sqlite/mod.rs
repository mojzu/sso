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

// TODO(feature): Implement SQLite driver, check unimplemented.

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
}
