use crate::driver::postgres::schema::auth_csrf;
use chrono::{DateTime, Utc};
use diesel::{prelude::*, result::QueryResult, PgConnection};
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
    pub fn create(conn: &PgConnection, value: &CsrfInsert) -> QueryResult<Csrf> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        diesel::insert_into(auth_csrf)
            .values(value)
            .get_result::<Csrf>(conn)
    }

    pub fn read_by_key(conn: &PgConnection, key: &str) -> QueryResult<Option<Csrf>> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        auth_csrf
            .filter(csrf_key.eq(key))
            .get_result::<Csrf>(conn)
            .optional()
    }

    pub fn delete_by_key(conn: &PgConnection, key: &str) -> QueryResult<usize> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        diesel::delete(auth_csrf.filter(csrf_key.eq(key))).execute(conn)
    }

    pub fn delete_by_ttl(conn: &PgConnection, now: &DateTime<Utc>) -> QueryResult<usize> {
        use crate::driver::postgres::schema::auth_csrf::dsl::*;

        diesel::delete(auth_csrf.filter(csrf_ttl.le(now))).execute(conn)
    }
}
