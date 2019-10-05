use crate::{driver::postgres::schema::sso_csrf, Csrf, CsrfCreate, CsrfDelete, DriverResult};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_csrf"]
#[primary_key(csrf_key)]
pub struct ModelCsrf {
    created_at: DateTime<Utc>,
    csrf_key: String,
    csrf_value: String,
    csrf_ttl: DateTime<Utc>,
    service_id: Uuid,
}

impl From<ModelCsrf> for Csrf {
    fn from(csrf: ModelCsrf) -> Self {
        Self {
            created_at: csrf.created_at,
            key: csrf.csrf_key,
            value: csrf.csrf_value,
            ttl: csrf.csrf_ttl,
            service_id: csrf.service_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "sso_csrf"]
struct ModelCsrfInsert<'a> {
    created_at: &'a DateTime<Utc>,
    csrf_key: &'a str,
    csrf_value: &'a str,
    csrf_ttl: &'a DateTime<Utc>,
    service_id: &'a Uuid,
}

impl<'a> ModelCsrfInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, create: &'a CsrfCreate) -> Self {
        Self {
            created_at: now,
            csrf_key: &create.key,
            csrf_value: &create.value,
            csrf_ttl: &create.ttl,
            service_id: &create.service_id,
        }
    }
}

impl ModelCsrf {
    pub fn create(conn: &PgConnection, create: &CsrfCreate) -> DriverResult<Csrf> {
        use crate::driver::postgres::schema::sso_csrf::dsl::*;

        let now = Utc::now();
        let value = ModelCsrfInsert::from_create(&now, create);
        diesel::insert_into(sso_csrf)
            .values(&value)
            .get_result::<ModelCsrf>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_opt(conn: &PgConnection, key: &str) -> DriverResult<Option<Csrf>> {
        use crate::driver::postgres::schema::sso_csrf::dsl::*;

        sso_csrf
            .filter(csrf_key.eq(key))
            .get_result::<ModelCsrf>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn delete(conn: &PgConnection, delete: &CsrfDelete) -> DriverResult<usize> {
        match delete {
            CsrfDelete::Key(key) => Self::delete_by_key(conn, key),
            CsrfDelete::Ttl(now) => Self::delete_by_ttl(conn, now),
        }
    }

    fn delete_by_key(conn: &PgConnection, key: &str) -> DriverResult<usize> {
        use crate::driver::postgres::schema::sso_csrf::dsl::*;

        diesel::delete(sso_csrf.filter(csrf_key.eq(key)))
            .execute(conn)
            .map_err(Into::into)
    }

    fn delete_by_ttl(conn: &PgConnection, now: &DateTime<Utc>) -> DriverResult<usize> {
        use crate::driver::postgres::schema::sso_csrf::dsl::*;

        diesel::delete(sso_csrf.filter(csrf_ttl.le(now)))
            .execute(conn)
            .map_err(Into::into)
    }
}
