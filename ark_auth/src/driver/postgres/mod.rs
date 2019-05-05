//! # PostgreSQL Driver
mod models;
mod schema;

use crate::{
    core::{Csrf, Key, Service, User},
    driver,
    driver::Error,
};
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
    pub fn initialise(database_url: &str) -> Result<Self, Error> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder().build(manager).map_err(Error::R2d2)?;
        let driver = Driver { pool };
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

impl driver::Driver for Driver {
    fn box_clone(&self) -> Box<driver::Driver> {
        Box::new((*self).clone())
    }

    fn key_list_where_id_lt(
        &self,
        key_service_id: i64,
        lt: i64,
        limit: i64,
    ) -> Result<Vec<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(service_id.eq(key_service_id).and(key_id.lt(lt)))
            .limit(limit)
            .order(key_id.asc())
            .load::<models::AuthKey>(&conn)
            .map_err(Error::Diesel)
            .map(|keys| {
                let keys: Vec<Key> = keys.into_iter().map(Into::into).collect();
                keys
            })
    }

    fn key_list_where_id_gt(
        &self,
        key_service_id: i64,
        gt: i64,
        limit: i64,
    ) -> Result<Vec<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(service_id.eq(key_service_id).and(key_id.gt(gt)))
            .limit(limit)
            .order(key_id.asc())
            .load::<models::AuthKey>(&conn)
            .map_err(Error::Diesel)
            .map(|keys| {
                let keys: Vec<Key> = keys.into_iter().map(Into::into).collect();
                keys
            })
    }

    fn key_create(
        &self,
        name: &str,
        value: &str,
        key_service_id: i64,
        key_user_id: Option<i64>,
    ) -> Result<Key, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        let value = models::AuthKeyInsert {
            key_name: name,
            key_value: value,
            service_id: key_service_id,
            user_id: key_user_id,
        };
        diesel::insert_into(auth_key)
            .values(&value)
            .get_result::<models::AuthKey>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn key_read_by_id(&self, id: i64) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(key_id.eq(id))
            .get_result::<models::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_service_value(&self, value: &str) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(key_value.eq(value).and(user_id.is_null()))
            .get_result::<models::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_user_value(
        &self,
        key_service_id: i64,
        value: &str,
    ) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.eq(key_service_id).and(user_id.is_not_null())),
            )
            .get_result::<models::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_user_id(
        &self,
        key_service_id: i64,
        key_user_id: i64,
    ) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(user_id.eq(key_user_id).and(service_id.eq(key_service_id)))
            // TODO(refactor): Better method to handle multiple keys?
            .order(created_at.asc())
            .get_result::<models::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_update_by_id(&self, id: i64, name: Option<&str>) -> Result<Key, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        let key_updated_at = chrono::Utc::now();
        let value = models::AuthKeyUpdate {
            updated_at: &key_updated_at,
            key_name: name,
        };
        diesel::update(auth_key.filter(key_id.eq(id)))
            .set(&value)
            .get_result::<models::AuthKey>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn key_delete_by_id(&self, id: i64) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_key.filter(key_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn service_create(&self, name: &str, url: &str) -> Result<Service, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        let value = models::AuthServiceInsert {
            service_name: name,
            service_url: url,
        };
        diesel::insert_into(auth_service)
            .values(&value)
            .get_result::<models::AuthService>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn service_read_by_id(&self, id: i64) -> Result<Option<Service>, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        auth_service
            .filter(service_id.eq(id))
            .get_result::<models::AuthService>(&conn)
            .map(|service| Some(service.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn service_update_by_id(&self, id: i64, name: Option<&str>) -> Result<Service, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        let service_updated_at = chrono::Utc::now();
        let value = models::AuthServiceUpdate {
            updated_at: &service_updated_at,
            service_name: name,
        };
        diesel::update(auth_service.filter(service_id.eq(id)))
            .set(&value)
            .get_result::<models::AuthService>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn service_delete_by_id(&self, id: i64) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_service.filter(service_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn user_list_where_id_lt(&self, lt: i64, limit: i64) -> Result<Vec<User>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .filter(user_id.lt(lt))
            .limit(limit)
            .order(user_id.asc())
            .load::<models::AuthUser>(&conn)
            .map_err(Error::Diesel)
            .map(|users| {
                let users: Vec<User> = users.into_iter().map(Into::into).collect();
                users
            })
    }

    fn user_list_where_id_gt(&self, gt: i64, limit: i64) -> Result<Vec<User>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .filter(user_id.gt(gt))
            .limit(limit)
            .order(user_id.asc())
            .load::<models::AuthUser>(&conn)
            .map_err(Error::Diesel)
            .map(|users| {
                let users: Vec<User> = users.into_iter().map(Into::into).collect();
                users
            })
    }

    fn user_create(
        &self,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
        password_revision: Option<i64>,
    ) -> Result<User, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let value = models::AuthUserInsert {
            user_name: name,
            user_email: email,
            user_password_hash: password_hash,
            user_password_revision: password_revision,
        };
        diesel::insert_into(auth_user)
            .values(&value)
            .get_result::<models::AuthUser>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn user_read_by_id(&self, id: i64) -> Result<Option<User>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .filter(user_id.eq(id))
            .get_result::<models::AuthUser>(&conn)
            .map(|user| Some(user.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn user_read_by_email(&self, email: &str) -> Result<Option<User>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .filter(user_email.eq(email))
            .get_result::<models::AuthUser>(&conn)
            .map(|user| Some(user.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn user_update_by_id(&self, id: i64, name: Option<&str>) -> Result<User, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let user_updated_at = chrono::Utc::now();
        let value = models::AuthUserUpdate {
            updated_at: &user_updated_at,
            user_name: name,
        };
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set(&value)
            .get_result::<models::AuthUser>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn user_update_password_by_id(
        &self,
        id: i64,
        password_hash: &str,
        password_revision: i64,
    ) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let user_updated_at = chrono::Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((
                updated_at.eq(user_updated_at),
                user_password_hash.eq(password_hash),
                user_password_revision.eq(password_revision),
            ))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn user_delete_by_id(&self, id: i64) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_user.filter(user_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn csrf_create(&self, key: &str, value: &str, csrf_service_id: i64) -> Result<Csrf, Error> {
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
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        auth_csrf
            .filter(csrf_key.eq(key))
            .get_result::<models::AuthCsrf>(&conn)
            .map(|csrf| Some(csrf.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn csrf_delete_by_key(&self, key: &str) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_csrf.filter(csrf_key.eq(key)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn csrf_delete_by_created_at(&self, csrf_created_at: &DateTime<Utc>) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_csrf.filter(created_at.le(csrf_created_at)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }
}
