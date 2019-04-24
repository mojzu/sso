use crate::db::{DbError, DbOrder};
use crate::models::{AuthService, AuthServiceInsert, AuthServiceUpdate};
use crate::schema;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;

pub fn list(
    offset: i64,
    limit: i64,
    order: DbOrder,
    service_service_id: i64,
    conn: &PgConnection,
) -> Result<Vec<AuthService>, DbError> {
    use crate::schema::auth_service::dsl::*;

    let filter_expr: Box<BoxableExpression<schema::auth_service::table, Pg, SqlType = Bool>> =
        match order {
            DbOrder::Asc => Box::new(service_id.gt(offset)),
            DbOrder::Desc => Box::new(service_id.lt(offset)),
        };
    let order_expr: Box<BoxableExpression<schema::auth_service::table, Pg, SqlType = ()>> =
        match order {
            DbOrder::Asc => Box::new(service_id.asc()),
            DbOrder::Desc => Box::new(service_id.desc()),
        };
    auth_service
        .filter(service_id.eq(service_service_id).and(filter_expr))
        .limit(limit)
        .order(order_expr)
        .load::<AuthService>(conn)
        .map_err(Into::into)
}

pub fn create(name: &str, url: &str, conn: &PgConnection) -> Result<AuthService, DbError> {
    use crate::schema::auth_service::dsl::*;

    let value = AuthServiceInsert {
        service_name: name,
        service_url: url,
    };
    diesel::insert_into(auth_service)
        .values(&value)
        .get_result::<AuthService>(conn)
        .map_err(Into::into)
}

pub fn read_by_id(
    id: i64,
    service_service_id: i64,
    conn: &PgConnection,
) -> Result<AuthService, DbError> {
    use crate::schema::auth_service::dsl::*;

    auth_service
        .filter(service_id.eq(id).and(service_id.eq(service_service_id)))
        .get_result::<AuthService>(conn)
        .map_err(Into::into)
}

pub fn update_by_id(
    id: i64,
    service_service_id: i64,
    name: Option<&str>,
    conn: &PgConnection,
) -> Result<AuthService, DbError> {
    use crate::schema::auth_service::dsl::*;

    let service_updated_at = chrono::Utc::now();
    let value = AuthServiceUpdate {
        updated_at: &service_updated_at,
        service_name: name,
    };
    diesel::update(auth_service.filter(service_id.eq(id).and(service_id.eq(service_service_id))))
        .set(&value)
        .get_result::<AuthService>(conn)
        .map_err(Into::into)
}

pub fn delete_by_id(
    id: i64,
    service_service_id: i64,
    conn: &PgConnection,
) -> Result<usize, DbError> {
    use crate::schema::auth_service::dsl::*;

    diesel::delete(auth_service.filter(service_id.eq(id).and(service_id.eq(service_service_id))))
        .execute(conn)
        .map_err(Into::into)
}
