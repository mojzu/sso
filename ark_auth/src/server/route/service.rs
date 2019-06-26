use crate::core;
use crate::core::audit::AuditMeta;
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{validate, Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;
use validator::Validate;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/service")
        .service(
            web::resource("")
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource("/{service_id}")
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler))
                .route(web::delete().to_async(delete_handler)),
        )
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ListQuery {
    #[validate(custom = "validate::id")]
    pub gt: Option<String>,
    #[validate(custom = "validate::id")]
    pub lt: Option<String>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<i64>,
}

impl FromJsonValue<ListQuery> for ListQuery {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListMetaResponse {
    pub gt: Option<String>,
    pub lt: Option<String>,
    pub limit: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub meta: ListMetaResponse,
    pub data: Vec<String>,
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = ListQuery::from_value(query.into_inner());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || list_inner(data.get_ref(), audit_meta, id, query))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn list_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    query: ListQuery,
) -> Result<ListResponse, Error> {
    core::key::authenticate_root(data.driver(), audit_meta, id)
        .and_then(|_| {
            let limit = query.limit.unwrap_or(10);
            let (gt, lt, services) = match query.lt {
                Some(lt) => {
                    let services = core::service::list_where_id_lt(data.driver(), &lt, limit)?;
                    (None, Some(lt), services)
                }
                None => {
                    let gt = query.gt.unwrap_or_else(|| "".to_owned());
                    let services = core::service::list_where_id_gt(data.driver(), &gt, limit)?;
                    (Some(gt), None, services)
                }
            };
            Ok(ListResponse {
                meta: ListMetaResponse { gt, lt, limit },
                data: services,
            })
        })
        .map_err(Into::into)
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct CreateBody {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(url)]
    pub url: String,
}

impl FromJsonValue<CreateBody> for CreateBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResponse {
    pub data: core::Service,
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = CreateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || create_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn create_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &CreateBody,
) -> Result<CreateResponse, Error> {
    core::key::authenticate_root(data.driver(), audit_meta, id)
        .and_then(|_| core::service::create(data.driver(), body.is_enabled, &body.name, &body.url))
        .map_err(Into::into)
        .map(|service| CreateResponse { data: service })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadResponse {
    pub data: core::Service,
}

fn read_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(String,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || read_inner(data.get_ref(), audit_meta, id, &path.0))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn read_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    service_id: &str,
) -> Result<ReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, _)| {
            core::service::read_by_id(data.driver(), service.as_ref(), service_id)
        })
        .map_err(Into::into)
        .and_then(|service| service.ok_or_else(|| Error::NotFound))
        .map(|service| ReadResponse { data: service })
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UpdateBody {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
}

impl FromJsonValue<UpdateBody> for UpdateBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResponse {
    pub data: core::Service,
}

fn update_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(String,)>,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = UpdateBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || update_inner(data.get_ref(), audit_meta, id, &path.0, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn update_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    service_id: &str,
    body: &UpdateBody,
) -> Result<UpdateResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, _)| {
            core::service::update_by_id(
                data.driver(),
                service.as_ref(),
                service_id,
                body.is_enabled,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|service| UpdateResponse { data: service })
}

fn delete_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    path: web::Path<(String,)>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || delete_inner(data.get_ref(), audit_meta, id, &path.0))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn delete_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    service_id: &str,
) -> Result<usize, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, _)| {
            core::service::delete_by_id(data.driver(), service.as_ref(), service_id)
        })
        .map_err(Into::into)
}
