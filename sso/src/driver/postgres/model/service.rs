use crate::{
    driver::postgres::schema::sso_service, DriverResult, Service, ServiceCreate, ServiceList,
    ServiceListQuery, ServiceRead, ServiceUpdate,
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
    user_allow_register: bool,
    user_email_text: String,
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
            user_allow_register: service.user_allow_register,
            user_email_text: service.user_email_text,
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
    user_allow_register: bool,
    user_email_text: &'a str,
    provider_local_url: Option<&'a str>,
    provider_github_oauth2_url: Option<&'a str>,
    provider_microsoft_oauth2_url: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "sso_service"]
struct ModelServiceUpdate<'a> {
    updated_at: &'a DateTime<Utc>,
    is_enabled: Option<bool>,
    name: Option<&'a str>,
    url: Option<&'a str>,
    user_allow_register: Option<bool>,
    user_email_text: Option<&'a str>,
    provider_local_url: Option<&'a str>,
    provider_github_oauth2_url: Option<&'a str>,
    provider_microsoft_oauth2_url: Option<&'a str>,
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
            ServiceListQuery::Limit => query
                .filter(sso_service::dsl::id.gt(Uuid::nil()))
                .limit(list.filter.limit)
                .order(sso_service::dsl::id.asc())
                .load::<ModelService>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            ServiceListQuery::IdGt(gt) => query
                .filter(sso_service::dsl::id.gt(gt))
                .limit(list.filter.limit)
                .order(sso_service::dsl::id.asc())
                .load::<ModelService>(conn)
                .map_err(Into::into)
                .map(|x| x.into_iter().map(|x| x.into()).collect()),
            ServiceListQuery::IdLt(lt) => query
                .filter(sso_service::dsl::id.lt(lt))
                .limit(list.filter.limit)
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
        let value = ModelServiceInsert {
            created_at: &now,
            updated_at: &now,
            id: &id,
            is_enabled: create.is_enabled,
            name: &create.name,
            url: &create.url,
            user_allow_register: create.user_allow_register,
            user_email_text: &create.user_email_text,
            provider_local_url: create.provider_local_url.as_ref().map(|x| &**x),
            provider_github_oauth2_url: create.provider_github_oauth2_url.as_ref().map(|x| &**x),
            provider_microsoft_oauth2_url: create
                .provider_microsoft_oauth2_url
                .as_ref()
                .map(|x| &**x),
        };
        diesel::insert_into(sso_service::table)
            .values(value)
            .get_result::<ModelService>(conn)
            .map_err(Into::into)
            .map(Into::into)
    }

    pub fn read(
        conn: &PgConnection,
        read: &ServiceRead,
        service_id: Option<Uuid>,
    ) -> DriverResult<Option<Service>> {
        match service_id {
            Some(service_id_mask) => sso_service::table
                .filter(
                    sso_service::dsl::id
                        .eq(read.id)
                        .and(sso_service::dsl::id.eq(service_id_mask)),
                )
                .get_result::<ModelService>(conn),
            None => sso_service::table
                .filter(sso_service::dsl::id.eq(read.id))
                .get_result::<ModelService>(conn),
        }
        .optional()
        .map_err(Into::into)
        .map(|x| x.map(Into::into))
    }

    pub fn update(conn: &PgConnection, update: &ServiceUpdate) -> DriverResult<Service> {
        let now = chrono::Utc::now();
        let value = ModelServiceUpdate {
            updated_at: &now,
            is_enabled: update.is_enabled,
            name: update.name.as_ref().map(|x| &**x),
            url: update.url.as_ref().map(|x| &**x),
            user_allow_register: update.user_allow_register,
            user_email_text: update.user_email_text.as_ref().map(|x| &**x),
            provider_local_url: update.provider_local_url.as_ref().map(|x| &**x),
            provider_github_oauth2_url: update.provider_github_oauth2_url.as_ref().map(|x| &**x),
            provider_microsoft_oauth2_url: update
                .provider_microsoft_oauth2_url
                .as_ref()
                .map(|x| &**x),
        };
        diesel::update(sso_service::table.filter(sso_service::dsl::id.eq(update.id)))
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
