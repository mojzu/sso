use crate::{
    driver::postgres::schema::auth_service, DriverResult, Service, ServiceCreate, ServiceUpdate,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, PgConnection};
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_service"]
#[primary_key(service_id)]
pub struct ModelService {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    service_id: Uuid,
    service_is_enabled: bool,
    service_name: String,
    service_url: String,
}

impl From<ModelService> for Service {
    fn from(service: ModelService) -> Self {
        Self {
            created_at: service.created_at,
            updated_at: service.updated_at,
            id: service.service_id,
            is_enabled: service.service_is_enabled,
            name: service.service_name,
            url: service.service_url,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_service"]
struct ModelServiceInsert<'a> {
    created_at: &'a DateTime<Utc>,
    updated_at: &'a DateTime<Utc>,
    service_id: &'a Uuid,
    service_is_enabled: bool,
    service_name: &'a str,
    service_url: &'a str,
}

impl<'a> ModelServiceInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a ServiceCreate) -> Self {
        Self {
            created_at: now,
            updated_at: now,
            service_id: id,
            service_is_enabled: create.is_enabled,
            service_name: create.name,
            service_url: create.url,
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "auth_service"]
struct ModelServiceUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    service_is_enabled: Option<bool>,
    service_name: Option<&'a str>,
}

impl<'a> ModelServiceUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a ServiceUpdate) -> Self {
        Self {
            updated_at: now,
            service_is_enabled: update.is_enabled,
            service_name: update.name,
        }
    }
}

impl ModelService {
    pub fn list_where_id_lt(conn: &PgConnection, lt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        auth_service
            .select(service_id)
            .filter(service_id.lt(lt))
            .limit(limit)
            .order(service_id.desc())
            .load::<Uuid>(conn)
            .map_err(Into::into)
            .map(|mut v| {
                v.reverse();
                v
            })
    }

    pub fn list_where_id_gt(conn: &PgConnection, gt: Uuid, limit: i64) -> DriverResult<Vec<Uuid>> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        auth_service
            .select(service_id)
            .filter(service_id.gt(gt))
            .limit(limit)
            .order(service_id.asc())
            .load::<Uuid>(conn)
            .map_err(Into::into)
    }

    pub fn create(conn: &PgConnection, create: &ServiceCreate) -> DriverResult<Service> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelServiceInsert::from_create(&now, &id, create);
        diesel::insert_into(auth_service)
            .values(value)
            .get_result::<ModelService>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_by_id(conn: &PgConnection, id: Uuid) -> DriverResult<Option<Service>> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        auth_service
            .filter(service_id.eq(id))
            .get_result::<ModelService>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn update_by_id(
        conn: &PgConnection,
        id: Uuid,
        update: &ServiceUpdate,
    ) -> DriverResult<Service> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        let now = chrono::Utc::now();
        let value = ModelServiceUpdate::from_update(&now, update);
        diesel::update(auth_service.filter(service_id.eq(id)))
            .set(value)
            .get_result::<ModelService>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn delete_by_id(conn: &PgConnection, id: Uuid) -> DriverResult<usize> {
        use crate::driver::postgres::schema::auth_service::dsl::*;

        diesel::delete(auth_service.filter(service_id.eq(id)))
            .execute(conn)
            .map_err(Into::into)
    }
}
