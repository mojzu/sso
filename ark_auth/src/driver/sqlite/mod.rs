//! # SQLite Driver
mod schema;

use crate::driver;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

embed_migrations!("migrations/sqlite");

#[derive(Clone)]
pub struct Driver {
    pool: r2d2::Pool<ConnectionManager<SqliteCOnnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<SqliteCOnnection>>;

impl Driver {
    pub fn initialise(database_url: &str) -> Result<Self, driver::Error> {
        let manager = ConnectionManager::<SqliteCOnnection>::new(database_url);
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
}
