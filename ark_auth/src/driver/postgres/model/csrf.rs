use crate::{driver::postgres::schema::auth_csrf, DriverError};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_csrf"]
#[primary_key(csrf_key)]
pub struct Csrf {
    pub created_at: DateTime<Utc>,
    pub csrf_key: String,
    pub csrf_value: String,
    pub csrf_ttl: DateTime<Utc>,
    pub service_id: Uuid,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_csrf"]
pub struct CsrfInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub csrf_key: &'a str,
    pub csrf_value: &'a str,
    pub csrf_ttl: &'a DateTime<Utc>,
    pub service_id: Uuid,
}

impl Csrf {
    pub fn create(
        conn: &PgConnection,
        key: &str,
        value: &str,
        ttl: &DateTime<Utc>,
        csrf_service_id: Uuid,
    ) -> Result<Csrf, DriverError> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        let now = Utc::now();
        let value = CsrfInsert {
            created_at: &now,
            csrf_key: key,
            csrf_value: value,
            csrf_ttl: ttl,
            service_id: csrf_service_id,
        };
        diesel::insert_into(auth_csrf)
            .values(&value)
            .get_result::<Csrf>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn read_by_key(conn: &PgConnection, key: &str) -> Result<Option<Csrf>, DriverError> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        auth_csrf
            .filter(csrf_key.eq(key))
            .get_result::<Csrf>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn delete_by_key(conn: &PgConnection, key: &str) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        diesel::delete(auth_csrf.filter(csrf_key.eq(key)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn delete_by_ttl(conn: &PgConnection, now: &DateTime<Utc>) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        diesel::delete(auth_csrf.filter(csrf_ttl.le(now)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }
}
