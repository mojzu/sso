//! # SQLite Driver
mod model;
mod schema;

use crate::driver::{Driver, Error};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

embed_migrations!("migrations/sqlite");

#[derive(Clone)]
pub struct SqliteDriver {
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

impl SqliteDriver {
    pub fn initialise(database_url: &str, max_connections: u32) -> Result<Self, Error> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .build(manager)
            .map_err(Error::R2d2)?;
        let driver = SqliteDriver { pool };
        driver.run_migrations()?;
        Ok(driver)
    }

    fn connection(&self) -> Result<PooledConnection, Error> {
        self.pool.get().map_err(Error::R2d2)
    }

    fn run_migrations(&self) -> Result<(), Error> {
        let connection = self.connection()?;
        embedded_migrations::run(&connection).map_err(Error::DieselMigrations)
    }
}

impl Driver for SqliteDriver {
    fn box_clone(&self) -> Box<Driver> {
        Box::new((*self).clone())
    }
}
