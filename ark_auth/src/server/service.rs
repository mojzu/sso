//! # Service
use crate::{
    core,
    server::{
        route_json_config, route_response_empty, route_response_json, validate_name,
        validate_unsigned, Data, Error, ValidateFromValue,
    },
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;
use validator::Validate;

/// List query.
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ListQuery {
    #[validate(custom = "validate_unsigned")]
    pub gt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    pub lt: Option<i64>,
    #[validate(custom = "validate_unsigned")]
    pub limit: Option<i64>,
}

impl ValidateFromValue<ListQuery> for ListQuery {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMetaResponse {
    pub gt: Option<i64>,
    pub lt: Option<i64>,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub meta: ListMetaResponse,
    pub data: Vec<core::Service>,
}

/// API version 1 service list route.
pub fn api_v1_list(
    data: web::Data<Data>,
    id: Identity,
    query: web::Query<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ListQuery::from_value(query.into_inner())
        .and_then(|query| list_inner(data, id, query))
        .then(|result| route_response_json(result))
}

fn list_inner(
    data: web::Data<Data>,
    id: Option<String>,
    query: ListQuery,
) -> impl Future<Item = ListResponse, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| {
                let limit = query.limit.unwrap_or(10);
                let (gt, lt, services) = match query.lt {
                    Some(lt) => {
                        let services =
                            core::service_list_where_id_lt(data.driver(), &service, lt, limit)?;
                        (None, Some(lt), services)
                    }
                    None => {
                        let gt = query.gt.unwrap_or(0);
                        let services =
                            core::service_list_where_id_gt(data.driver(), &service, gt, limit)?;
                        (Some(gt), None, services)
                    }
                };

                Ok(ListResponse {
                    meta: ListMetaResponse { gt, lt, limit },
                    data: services,
                })
            })
            .map_err(Into::into)
    })
    .map_err(Into::into)
}

// TODO(feature): Create new services via API.
// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// #[serde(deny_unknown_fields)]
// pub struct CreateBody {
//     #[validate(custom = "validate_name")]
//     pub name: String,
//     #[validate(url)]
//     pub url: String,
// }
// impl ValidateFromValue<CreateBody> for CreateBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResponse {
    data: core::Service,
}

/// API version 1 service read route.
pub fn api_v1_read(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    read_inner(data, id, path.0).then(|result| route_response_json(result))
}

fn read_inner(
    data: web::Data<Data>,
    id: Option<String>,
    service_id: i64,
) -> impl Future<Item = ReadResponse, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::service_read_by_id(data.driver(), &service, service_id))
            .map_err(Into::into)
    })
    .map_err(Into::into)
    .map(|service| ReadResponse { data: service })
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateBody {
    #[validate(custom = "validate_name")]
    pub name: Option<String>,
}

impl ValidateFromValue<UpdateBody> for UpdateBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResponse {
    data: core::Service,
}

/// API version 1 service update route.
pub fn api_v1_update(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    UpdateBody::from_value(body.into_inner())
        .and_then(move |body| update_inner(data, id, path.0, body))
        .then(|result| route_response_json(result))
}

fn update_inner(
    data: web::Data<Data>,
    id: Option<String>,
    service_id: i64,
    body: UpdateBody,
) -> impl Future<Item = UpdateResponse, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| {
                core::service_update_by_id(
                    data.driver(),
                    &service,
                    service_id,
                    body.name.as_ref().map(|x| &**x),
                )
            })
            .map_err(Into::into)
    })
    .map_err(Into::into)
    .map(|service| UpdateResponse { data: service })
}

/// API version 1 service delete route.
pub fn api_v1_delete(
    data: web::Data<Data>,
    id: Identity,
    path: web::Path<(i64,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    delete_inner(data, id, path.0).then(|result| route_response_empty(result))
}

fn delete_inner(
    data: web::Data<Data>,
    id: Option<String>,
    service_id: i64,
) -> impl Future<Item = usize, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::service_delete_by_id(data.driver(), &service, service_id))
            .map_err(Into::into)
    })
    .map_err(Into::into)
}

/// API version 1 service scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/service")
        .service(
            web::resource("")
                .route(web::get().to_async(api_v1_list))
                // TODO(feature): Create new services via API.
                // .route(
                //     web::post()
                //         .data(route_json_config())
                //         .to_async(api_v1_create),
                // ),
        )
        .service(
            web::resource("/{service_id}")
                .route(web::get().to_async(api_v1_read))
                .route(
                    web::patch()
                        .data(route_json_config())
                        .to_async(api_v1_update),
                )
                .route(web::delete().to_async(api_v1_delete)),
        )
}
