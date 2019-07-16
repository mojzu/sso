use crate::core;
use crate::core::{AuditMeta, ServiceQuery};
use crate::server::api::{
    path, ServiceCreateBody, ServiceListQuery, ServiceListResponse, ServiceReadResponse,
    ServiceUpdateBody,
};
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::SERVICE)
        .service(
            web::resource(path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(
            web::resource(path::ID)
                .route(web::get().to_async(read_handler))
                .route(web::patch().to_async(update_handler))
                .route(web::delete().to_async(delete_handler)),
        )
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = ServiceListQuery::from_value(query.into_inner());

    audit_meta
        .join(query)
        .and_then(|(audit_meta, query)| {
            web::block(move || list_inner(data.get_ref(), audit_meta, id, query.into()))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn list_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    query: ServiceQuery,
) -> Result<ServiceListResponse, Error> {
    core::key::authenticate_root(data.driver(), audit_meta, id)
        .and_then(|mut audit| {
            let service_ids = core::service::list(data.driver(), &mut audit, &query)?;
            Ok(ServiceListResponse {
                meta: query,
                data: service_ids,
            })
        })
        .map_err(Into::into)
}

fn create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = ServiceCreateBody::from_value(body.into_inner());

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
    body: &ServiceCreateBody,
) -> Result<ServiceReadResponse, Error> {
    core::key::authenticate_root(data.driver(), audit_meta, id)
        .and_then(|mut audit| {
            core::service::create(
                data.driver(),
                &mut audit,
                body.is_enabled,
                &body.name,
                &body.url,
            )
        })
        .map_err(Into::into)
        .map(|service| ServiceReadResponse { data: service })
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
) -> Result<ServiceReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::service::read_by_id(data.driver(), service.as_ref(), &mut audit, service_id)
        })
        .map_err(Into::into)
        .and_then(|service| service.ok_or_else(|| Error::NotFound))
        .map(|service| ServiceReadResponse { data: service })
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
    let body = ServiceUpdateBody::from_value(body.into_inner());

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
    body: &ServiceUpdateBody,
) -> Result<ServiceReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::service::update_by_id(
                data.driver(),
                service.as_ref(),
                &mut audit,
                service_id,
                body.is_enabled,
                body.name.as_ref().map(|x| &**x),
            )
        })
        .map_err(Into::into)
        .map(|service| ServiceReadResponse { data: service })
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
        .and_then(|(service, mut audit)| {
            core::service::delete_by_id(data.driver(), service.as_ref(), &mut audit, service_id)
        })
        .map_err(Into::into)
}
