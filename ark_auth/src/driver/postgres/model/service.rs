use crate::{driver::postgres::schema::auth_service, DriverError};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_service"]
#[primary_key(service_id)]
pub struct Service {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub service_id: Uuid,
    pub service_is_enabled: bool,
    pub service_name: String,
    pub service_url: String,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_service"]
pub struct ServiceInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub updated_at: &'a DateTime<Utc>,
    pub service_id: Uuid,
    pub service_is_enabled: bool,
    pub service_name: &'a str,
    pub service_url: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "auth_service"]
pub struct ServiceUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub service_is_enabled: Option<bool>,
    pub service_name: Option<&'a str>,
}

impl Service {
    pub fn list_where_id_lt(
        conn: &PgConnection,
        lt: Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        auth_service
            .select(service_id)
            .filter(service_id.lt(lt))
            .limit(limit)
            .order(service_id.desc())
            .load::<Uuid>(conn)
            .map_err(DriverError::Diesel)
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    pub fn list_where_id_gt(
        conn: &PgConnection,
        gt: Uuid,
        limit: i64,
    ) -> Result<Vec<Uuid>, DriverError> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        auth_service
            .select(service_id)
            .filter(service_id.gt(gt))
            .limit(limit)
            .order(service_id.asc())
            .load::<Uuid>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn create(
        conn: &PgConnection,
        is_enabled: bool,
        name: &str,
        url: &str,
    ) -> Result<Service, DriverError> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let now = Utc::now();
        let value = ServiceInsert {
            created_at: &now,
            updated_at: &now,
            service_id: Uuid::new_v4(),
            service_is_enabled: is_enabled,
            service_name: name,
            service_url: url,
        };
        diesel::insert_into(auth_service)
            .values(&value)
            .get_result::<Service>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn read_by_id(conn: &PgConnection, id: Uuid) -> Result<Option<Service>, DriverError> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        auth_service
            .filter(service_id.eq(id))
            .get_result::<Service>(conn)
            .map(Some)
            .or_else(|err| match err {
                diesel::result::Error::NotFound => Ok(None),
                _ => Err(DriverError::Diesel(err)),
            })
    }

    pub fn update_by_id(
        conn: &PgConnection,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<&str>,
    ) -> Result<Service, DriverError> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let now = chrono::Utc::now();
        let value = ServiceUpdate {
            updated_at: &now,
            service_is_enabled: is_enabled,
            service_name: name,
        };
        diesel::update(auth_service.filter(service_id.eq(id)))
            .set(&value)
            .get_result::<Service>(conn)
            .map_err(DriverError::Diesel)
    }

    pub fn delete_by_id(conn: &PgConnection, id: Uuid) -> Result<usize, DriverError> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        diesel::delete(auth_service.filter(service_id.eq(id)))
            .execute(conn)
            .map_err(DriverError::Diesel)
    }
}
