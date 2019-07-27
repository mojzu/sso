//! # SQLite Driver
mod model;
mod schema;

use crate::core::{Disk, DiskOptions};
use crate::driver::{Driver, Error};
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use std::convert::TryInto;

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

    fn uuid() -> String {
        uuid::Uuid::new_v4().to_simple().to_string()
    }

    fn run_migrations(&self) -> Result<(), Error> {
        let connection = self.connection()?;
        embedded_migrations::run(&connection).map_err(Error::DieselMigrations)
    }

    fn disk_list_where_name_gte_inner(
        &self,
        name_gte: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<String>, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        kv_disk
            .select(disk_id)
            .filter(disk_name.ge(name_gte))
            .limit(limit)
            .offset(offset)
            .order(disk_name.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }
}

impl Driver for SqliteDriver {
    fn box_clone(&self) -> Box<Driver> {
        Box::new((*self).clone())
    }

    fn disk_list_where_name_gte(
        &self,
        name_gte: &str,
        offset_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<String>, Error> {
        let offset: i64 = if offset_id.is_some() { 1 } else { 0 };
        self.disk_list_where_name_gte_inner(name_gte, limit, offset)
            .and_then(|res| {
                if let Some(offset_id) = offset_id {
                    for (i, id) in res.iter().enumerate() {
                        if id == offset_id {
                            let offset: i64 = (i + 1).try_into().unwrap();
                            return self.disk_list_where_name_gte_inner(name_gte, limit, offset);
                        }
                    }
                }
                Ok(res)
            })
    }

    fn disk_create(&self, name: &str, options: &DiskOptions) -> Result<Disk, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now().to_rfc3339();
        let id = SqliteDriver::uuid();
        let options = serde_json::to_string(options).unwrap();
        let value = model::DiskInsert {
            created_at: &now,
            updated_at: &now,
            disk_id: &id,
            disk_name: name,
            disk_options: &options,
        };
        diesel::insert_into(kv_disk)
            .values(&value)
            .execute(&conn)
            .map_err(Error::Diesel)?;
        self.disk_read_by_id(&id)
    }

    fn disk_read_by_id(&self, id: &str) -> Result<Disk, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        kv_disk
            .filter(disk_id.eq(id))
            .get_result::<model::Disk>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn disk_read_by_name(&self, name: &str) -> Result<Disk, Error> {
        use crate::driver::sqlite::schema::kv_disk::dsl::*;

        let conn = self.connection()?;
        kv_disk
            .filter(disk_name.eq(name))
            .get_result::<model::Disk>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }
}
