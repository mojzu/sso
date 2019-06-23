//! # PostgreSQL Driver
mod model;
mod schema;

use crate::core::{Audit, Csrf, Key, Service, User};
use crate::driver;
use crate::driver::Error;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use serde_json::Value;

embed_migrations!("migrations/postgres");

#[derive(Clone)]
pub struct Driver {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

impl Driver {
    pub fn initialise(database_url: &str, max_connections: u32) -> Result<Self, Error> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = r2d2::Pool::builder()
            .max_size(max_connections)
            .build(manager)
            .map_err(Error::R2d2)?;
        let driver = Driver { pool };
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
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        match service_id_mask {
            Some(service_id_mask) => auth_key
                .select(key_id)
                .filter(service_id.eq(service_id_mask).and(key_id.lt(lt)))
                .limit(limit)
                .order(key_id.asc())
                .load::<String>(&conn),
            None => auth_key
                .select(key_id)
                .filter(key_id.lt(lt))
                .limit(limit)
                .order(key_id.asc())
                .load::<String>(&conn),
        }
        .map_err(Error::Diesel)
    }

    fn key_list_where_id_gt(
        &self,
        gt: &str,
        limit: i64,
        service_id_mask: Option<&str>,
    ) -> Result<Vec<String>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        match service_id_mask {
            Some(service_id_mask) => auth_key
                .select(key_id)
                .filter(service_id.eq(service_id_mask).and(key_id.gt(gt)))
                .limit(limit)
                .order(key_id.asc())
                .load::<String>(&conn),
            None => auth_key
                .select(key_id)
                .filter(key_id.gt(gt))
                .limit(limit)
                .order(key_id.asc())
                .load::<String>(&conn),
        }
        .map_err(Error::Diesel)
    }

    fn key_create(
        &self,
        is_active: bool,
        name: &str,
        value: &str,
        key_service_id: Option<&str>,
        key_user_id: Option<&str>,
    ) -> Result<Key, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now();
        let id = Driver::uuid();
        let value = model::AuthKeyInsert {
            created_at: &now,
            updated_at: &now,
            key_id: &id,
            key_is_active: is_active,
            key_name: name,
            key_value: value,
            service_id: key_service_id,
            user_id: key_user_id,
        };
        diesel::insert_into(auth_key)
            .values(&value)
            .get_result::<model::AuthKey>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn key_read_by_id(&self, id: &str) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(key_id.eq(id))
            .get_result::<model::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_user_id(
        &self,
        key_service_id: &str,
        key_user_id: &str,
    ) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            // TODO(refactor): Better method to handle multiple keys?
            .filter(
                user_id
                    .eq(key_user_id)
                    .and(service_id.eq(key_service_id))
                    .and(key_is_active.eq(true)),
            )
            .order(created_at.asc())
            .get_result::<model::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_root_value(&self, value: &str) -> Result<Option<Key>, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        auth_key
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.is_null())
                    .and(user_id.is_null()),
            )
            .get_result::<model::AuthKey>(&conn)
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
            .filter(
                key_value
                    .eq(value)
                    .and(service_id.is_not_null())
                    .and(user_id.is_null()),
            )
            .get_result::<model::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_read_by_user_value(
        &self,
        key_service_id: &str,
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
            .get_result::<model::AuthKey>(&conn)
            .map(|key| Some(key.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn key_update_by_id(
        &self,
        id: &str,
        is_active: Option<bool>,
        name: Option<&str>,
    ) -> Result<Key, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now();
        let value = model::AuthKeyUpdate {
            updated_at: &now,
            key_is_active: is_active,
            key_name: name,
        };
        diesel::update(auth_key.filter(key_id.eq(id)))
            .set(&value)
            .get_result::<model::AuthKey>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn key_update_many_by_user_id(
        &self,
        key_user_id: &str,
        is_active: Option<bool>,
        name: Option<&str>,
    ) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now();
        let value = model::AuthKeyUpdate {
            updated_at: &now,
            key_is_active: is_active,
            key_name: name,
        };
        diesel::update(auth_key.filter(user_id.eq(key_user_id)))
            .set(&value)
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn key_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_key.filter(key_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn key_delete_root(&self) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_key::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_key.filter(service_id.is_null().and(user_id.is_null())))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn service_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        auth_service
            .select(service_id)
            .filter(service_id.lt(lt))
            .limit(limit)
            .order(service_id.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn service_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        auth_service
            .select(service_id)
            .filter(service_id.gt(gt))
            .limit(limit)
            .order(service_id.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn service_create(&self, is_active: bool, name: &str, url: &str) -> Result<Service, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now();
        let id = Driver::uuid();
        let value = model::AuthServiceInsert {
            created_at: &now,
            updated_at: &now,
            service_id: &id,
            service_is_active: is_active,
            service_name: name,
            service_url: url,
        };
        diesel::insert_into(auth_service)
            .values(&value)
            .get_result::<model::AuthService>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn service_read_by_id(&self, id: &str) -> Result<Option<Service>, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        auth_service
            .filter(service_id.eq(id))
            .get_result::<model::AuthService>(&conn)
            .map(|service| Some(service.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn service_update_by_id(
        &self,
        id: &str,
        is_active: Option<bool>,
        name: Option<&str>,
    ) -> Result<Service, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now();
        let value = model::AuthServiceUpdate {
            updated_at: &now,
            service_is_active: is_active,
            service_name: name,
        };
        diesel::update(auth_service.filter(service_id.eq(id)))
            .set(&value)
            .get_result::<model::AuthService>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn service_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_service.filter(service_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn user_list_where_id_lt(&self, lt: &str, limit: i64) -> Result<Vec<String>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .select(user_id)
            .filter(user_id.lt(lt))
            .limit(limit)
            .order(user_id.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn user_list_where_id_gt(&self, gt: &str, limit: i64) -> Result<Vec<String>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .select(user_id)
            .filter(user_id.gt(gt))
            .limit(limit)
            .order(user_id.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn user_list_where_email_eq(&self, email_eq: &str, limit: i64) -> Result<Vec<String>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .select(user_id)
            .filter(user_email.eq(email_eq))
            .limit(limit)
            .order(user_id.asc())
            .load::<String>(&conn)
            .map_err(Error::Diesel)
    }

    fn user_create(
        &self,
        is_active: bool,
        name: &str,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now();
        let id = Driver::uuid();
        let value = model::AuthUserInsert {
            created_at: &now,
            updated_at: &now,
            user_id: &id,
            user_is_active: is_active,
            user_name: name,
            user_email: email,
            user_password_hash: password_hash,
        };
        diesel::insert_into(auth_user)
            .values(&value)
            .get_result::<model::AuthUser>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn user_read_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        auth_user
            .filter(user_id.eq(id))
            .get_result::<model::AuthUser>(&conn)
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
            .get_result::<model::AuthUser>(&conn)
            .map(|user| Some(user.into()))
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(Error::Diesel(err)),
            })
    }

    fn user_update_by_id(
        &self,
        id: &str,
        is_active: Option<bool>,
        name: Option<&str>,
    ) -> Result<User, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now();
        let value = model::AuthUserUpdate {
            updated_at: &now,
            user_is_active: is_active,
            user_name: name,
        };
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set(&value)
            .get_result::<model::AuthUser>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn user_update_email_by_id(&self, id: &str, email: &str) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((updated_at.eq(now), user_email.eq(email)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn user_update_password_by_id(&self, id: &str, password_hash: &str) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        let now = chrono::Utc::now();
        diesel::update(auth_user.filter(user_id.eq(id)))
            .set((updated_at.eq(now), user_password_hash.eq(password_hash)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn user_delete_by_id(&self, id: &str) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_user::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_user.filter(user_id.eq(id)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn csrf_create(
        &self,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        csrf_service_id: &str,
    ) -> Result<Csrf, Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        let now = Utc::now();
        let value = model::AuthCsrfInsert {
            created_at: &now,
            csrf_key: key,
            csrf_value: value,
            csrf_ttl: ttl,
            service_id: csrf_service_id,
        };
        diesel::insert_into(auth_csrf)
            .values(&value)
            .get_result::<model::AuthCsrf>(&conn)
            .map_err(Error::Diesel)
            .map(Into::into)
    }

    fn csrf_read_by_key(&self, key: &str) -> Result<Option<Csrf>, Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        auth_csrf
            .filter(csrf_key.eq(key))
            .get_result::<model::AuthCsrf>(&conn)
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

    fn csrf_delete_by_ttl(&self, now: &DateTime<Utc>) -> Result<usize, Error> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let conn = self.connection()?;
        diesel::delete(auth_csrf.filter(csrf_ttl.le(now)))
            .execute(&conn)
            .map_err(Error::Diesel)
    }

    fn audit_create(
        &self,
        _user_agent: &str,
        _remote: &str,
        _forwarded_for: Option<&str>,
        _key: &str,
        _data: &Value,
        _key_id: &str,
        _service_id: Option<&str>,
        _user_id: Option<&str>,
        _user_key_id: Option<&str>,
    ) -> Result<Audit, Error> {
        unimplemented!();
    }

    fn audit_delete_by_created_at(&self, _created_at: &DateTime<Utc>) -> Result<usize, Error> {
        unimplemented!();
    }
}
