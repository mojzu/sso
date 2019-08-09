mod model;
mod schema;

use crate::core::{Audit, Csrf, Key, Service, User};
use crate::driver::{Driver, DriverError};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use serde_json::Value;

embed_migrations!("migrations/sqlite");

/// ## SQLite Driver Implementation
#[derive(Clone)]
pub struct SqliteDriver {
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

// TODO(refactor): Implement SQLite driver, check unimplemented.

impl SqliteDriver {
    /// Initialise driver with connection URL and number of pooled connections.
    pub fn initialise(database_url: &str, max_connections: u32) -> Result<Self, DriverError> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .build(manager)
            .map_err(DriverError::R2d2)?;
        let driver = SqliteDriver { pool };
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

impl Driver for SqliteDriver {
    fn box_clone(&self) -> Box<Driver> {
        Box::new((*self).clone())
    }
}
