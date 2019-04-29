use crate::db::DbError;
use crate::models::{AuthKey, AuthKeyInsert, AuthKeyUpdate};
use crate::schema;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use uuid::Uuid;

pub fn list(
    gt: Option<i64>,
    lt: Option<i64>,
    limit: i64,
    key_service_id: i64,
    conn: &PgConnection,
) -> Result<Vec<AuthKey>, DbError> {
    use crate::schema::auth_key::dsl::*;

    let filter_expr: Box<BoxableExpression<schema::auth_key::table, Pg, SqlType = Bool>> = match lt
    {
        Some(lt) => Box::new(key_id.lt(lt)),
        None => Box::new(key_id.gt(gt.unwrap_or(0))),
    };
    auth_key
        .filter(service_id.eq(key_service_id).and(filter_expr))
        .limit(limit)
        .order(key_id.asc())
        .load::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn create(
    name: &str,
    key_service_id: i64,
    key_user_id: Option<i64>,
    conn: &PgConnection,
) -> Result<AuthKey, DbError> {
    use crate::schema::auth_key::dsl::*;

    let uuid_value = Uuid::new_v4().to_simple().to_string();
    let value = AuthKeyInsert {
        key_name: name,
        key_value: &uuid_value,
        service_id: key_service_id,
        user_id: key_user_id,
    };

    diesel::insert_into(auth_key)
        .values(&value)
        .get_result::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn read_by_id(id: i64, key_service_id: i64, conn: &PgConnection) -> Result<AuthKey, DbError> {
    use crate::schema::auth_key::dsl::*;

    auth_key
        .filter(key_id.eq(id).and(service_id.eq(key_service_id)))
        .get_result::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn service_read_by_value(value: &str, conn: &PgConnection) -> Result<AuthKey, DbError> {
    use crate::schema::auth_key::dsl::*;

    auth_key
        .filter(key_value.eq(value).and(user_id.is_null()))
        .get_result::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn user_read_by_value(
    value: &str,
    key_service_id: i64,
    conn: &PgConnection,
) -> Result<AuthKey, DbError> {
    use crate::schema::auth_key::dsl::*;

    auth_key
        .filter(
            key_value
                .eq(value)
                .and(service_id.eq(key_service_id).and(user_id.is_not_null())),
        )
        .get_result::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn read_by_user_id(
    key_user_id: i64,
    key_service_id: i64,
    conn: &PgConnection,
) -> Result<AuthKey, DbError> {
    use crate::schema::auth_key::dsl::*;

    auth_key
        .filter(user_id.eq(key_user_id).and(service_id.eq(key_service_id)))
        .order(created_at.asc())
        .get_result::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn update_by_id(
    id: i64,
    key_service_id: i64,
    name: Option<&str>,
    conn: &PgConnection,
) -> Result<AuthKey, DbError> {
    use crate::schema::auth_key::dsl::*;

    let key_updated_at = chrono::Utc::now();
    let value = AuthKeyUpdate {
        updated_at: &key_updated_at,
        key_name: name,
    };
    diesel::update(auth_key.filter(key_id.eq(id).and(key_id.eq(key_service_id))))
        .set(&value)
        .get_result::<AuthKey>(conn)
        .map_err(Into::into)
}

pub fn delete_by_id(id: i64, key_service_id: i64, conn: &PgConnection) -> Result<usize, DbError> {
    use crate::schema::auth_key::dsl::*;

    diesel::delete(auth_key.filter(key_id.eq(id).and(service_id.eq(key_service_id))))
        .execute(conn)
        .map_err(Into::into)
}
