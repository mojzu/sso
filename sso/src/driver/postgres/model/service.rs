use crate::{
    driver::postgres::schema::sso_service, DriverResult, Service, ServiceCreate, ServiceList,
    ServiceListQuery, ServiceUpdate,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "sso_service"]
#[primary_key(id)]
pub struct ModelService {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    id: Uuid,
    is_enabled: bool,
    name: String,
    url: String,
    provider_local_url: Option<String>,
    provider_github_oauth2_url: Option<String>,
    provider_microsoft_oauth2_url: Option<String>,
}

impl From<ModelService> for Service {
    fn from(service: ModelService) -> Self {
        Self {
            created_at: service.created_at,
            updated_at: service.updated_at,
            id: service.id,
            is_enabled: service.is_enabled,
            name: service.name,
            url: service.url,
            provider_local_url: service.provider_local_url,
            provider_github_oauth2_url: service.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: service.provider_microsoft_oauth2_url,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "sso_service"]
struct ModelServiceInsert<'a> {
    created_at: &'a DateTime<Utc>,
    updated_at: &'a DateTime<Utc>,
    id: &'a Uuid,
    is_enabled: bool,
    name: &'a str,
    url: &'a str,
    provider_local_url: Option<&'a str>,
    provider_github_oauth2_url: Option<&'a str>,
    provider_microsoft_oauth2_url: Option<&'a str>,
}

impl<'a> ModelServiceInsert<'a> {
    fn from_create(now: &'a DateTime<Utc>, id: &'a Uuid, create: &'a ServiceCreate) -> Self {
        Self {
            created_at: now,
            updated_at: now,
            id,
            is_enabled: create.is_enabled,
            name: &create.name,
            url: &create.url,
            provider_local_url: create.provider_local_url.as_ref().map(|x| &**x),
            provider_github_oauth2_url: create.provider_github_oauth2_url.as_ref().map(|x| &**x),
            provider_microsoft_oauth2_url: create
                .provider_microsoft_oauth2_url
                .as_ref()
                .map(|x| &**x),
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "sso_service"]
struct ModelServiceUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    is_enabled: Option<bool>,
    name: Option<&'a str>,
    url: Option<&'a str>,
    provider_local_url: Option<&'a str>,
    provider_github_oauth2_url: Option<&'a str>,
    provider_microsoft_oauth2_url: Option<&'a str>,
}

impl<'a> ModelServiceUpdate<'a> {
    fn from_update(now: &'a DateTime<Utc>, update: &'a ServiceUpdate) -> Self {
        Self {
            updated_at: now,
            is_enabled: update.is_enabled,
            name: update.name.as_ref().map(|x| &**x),
            url: update.url.as_ref().map(|x| &**x),
            provider_local_url: update.provider_local_url.as_ref().map(|x| &**x),
            provider_github_oauth2_url: update.provider_github_oauth2_url.as_ref().map(|x| &**x),
            provider_microsoft_oauth2_url: update
                .provider_microsoft_oauth2_url
                .as_ref()
                .map(|x| &**x),
        }
    }
}

impl ModelService {
    pub fn list(conn: &PgConnection, list: &ServiceList) -> DriverResult<Vec<Service>> {
        use diesel::dsl::any;

        let mut query = sso_service::table.into_boxed();

        if let Some(id) = &list.filter.id {
            let id: Vec<Uuid> = id.iter().copied().collect();
            query = query.filter(sso_service::dsl::id.eq(any(id)));
        }
        if let Some(is_enabled) = list.filter.is_enabled {
            query = query.filter(sso_service::dsl::is_enabled.eq(is_enabled));
        }

        match list.query {
            ServiceListQuery::Limit(limit) => query
                .filter(sso_service::dsl::id.gt(Uuid::nil()))
                .limit(*limit)
                .order(sso_service::dsl::id.asc())
                .load::<ModelService>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            ServiceListQuery::IdGt(gt, limit) => query
                .filter(sso_service::dsl::id.gt(gt))
                .limit(*limit)
                .order(sso_service::dsl::id.asc())
                .load::<ModelService>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            ServiceListQuery::IdLt(lt, limit) => query
                .filter(sso_service::dsl::id.lt(lt))
                .limit(*limit)
                .order(sso_service::dsl::id.desc())
                .load::<ModelService>(conn)
                .map_err(Into::into)
                .map(|mut x| {
                    x.reverse();
                    x.into_iter().map(|x| x.into()).collect()
                }),
        }
    }

    pub fn create(conn: &PgConnection, create: &ServiceCreate) -> DriverResult<Service> {
        let now = Utc::now();
        let id = Uuid::new_v4();
        let value = ModelServiceInsert::from_create(&now, &id, create);
        diesel::insert_into(sso_service::table)
            .values(value)
            .get_result::<ModelService>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read_opt(conn: &PgConnection, id: &Uuid) -> DriverResult<Option<Service>> {
        sso_service::table
            .filter(sso_service::dsl::id.eq(id))
            .get_result::<ModelService>(conn)
            .optional()
            .map_err(Into::into)
            .map(|x| x.map(Into::into))
    }

    pub fn update(conn: &PgConnection, id: &Uuid, update: &ServiceUpdate) -> DriverResult<Service> {
        let now = chrono::Utc::now();
        let value = ModelServiceUpdate::from_update(&now, update);
        diesel::update(sso_service::table.filter(sso_service::dsl::id.eq(id)))
            .set(value)
            .get_result::<ModelService>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn delete(conn: &PgConnection, id: &Uuid) -> DriverResult<usize> {
        diesel::delete(sso_service::table.filter(sso_service::dsl::id.eq(id)))
            .execute(conn)
            .map_err(Into::into)
    }
}
