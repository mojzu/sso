use crate::db::DbError;
use crate::models::{AuthCsrf, AuthCsrfInsert};
use chrono::Utc;
use diesel::prelude::*;
use time::Duration;

/// Create one CSRF key, value pair. Key must be unique.
pub fn create(
    key: &str,
    value: &str,
    csrf_service_id: i64,
    conn: &PgConnection,
) -> Result<AuthCsrf, DbError> {
    use crate::schema::auth_csrf::dsl::*;

    delete_by_age(conn)?;

    let value = AuthCsrfInsert {
        csrf_key: key,
        csrf_value: value,
        service_id: csrf_service_id,
    };
    diesel::insert_into(auth_csrf)
        .values(&value)
        .get_result::<AuthCsrf>(conn)
        .map_err(Into::into)
}

/// Read one CSRF key, value pair. CSRF key, value pair is deleted after one read.
pub fn read_by_key(key: &str, conn: &PgConnection) -> Result<AuthCsrf, DbError> {
    use crate::schema::auth_csrf::dsl::*;

    delete_by_age(conn)?;

    let csrf: Result<AuthCsrf, DbError> = auth_csrf
        .filter(csrf_key.eq(key))
        .get_result::<AuthCsrf>(conn)
        .map_err(Into::into);
    let csrf = csrf?;

    delete_by_key(key, conn)?;
    Ok(csrf)
}

/// Delete one CSRF key, value pair.
fn delete_by_key(key: &str, conn: &PgConnection) -> Result<usize, DbError> {
    use crate::schema::auth_csrf::dsl::*;

    diesel::delete(auth_csrf.filter(csrf_key.eq(key)))
        .execute(conn)
        .map_err(Into::into)
}

/// Delete many CSRF key, value pairs created more than one hour ago.
fn delete_by_age(conn: &PgConnection) -> Result<usize, DbError> {
    use crate::schema::auth_csrf::dsl::*;

    let previous_hour = Utc::now() - Duration::hours(1);
    diesel::delete(auth_csrf.filter(created_at.le(previous_hour)))
        .execute(conn)
        .map_err(Into::into)
}
