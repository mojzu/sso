use crate::core;
use crate::core::{AuditMeta, AuditQuery};
use crate::server::api::{
    path, AuditCreateBody, AuditCreateResponse, AuditListQuery, AuditListResponse,
    AuditReadResponse,
};
use crate::server::route::{request_audit_meta, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::AUDIT)
        .service(
            web::resource(path::NONE)
                .route(web::get().to_async(list_handler))
                .route(web::post().to_async(create_handler)),
        )
        .service(web::resource(path::ID).route(web::get().to_async(read_handler)))
}

fn list_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    query: web::Query<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let query = AuditListQuery::from_value(query.into_inner());

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
    query: AuditQuery,
) -> Result<AuditListResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            let audit_ids = core::audit::list(data.driver(), service.as_ref(), &mut audit, &query)?;
            Ok(AuditListResponse {
                meta: query,
                data: audit_ids,
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
    let body = AuditCreateBody::from_value(body.into_inner());

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
    body: &AuditCreateBody,
) -> Result<AuditCreateResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(move |(_, mut audit)| {
            audit
                .set_user_id(body.user_id.to_owned())
                .set_user_key_id(body.user_key_id.to_owned())
                .create(data.driver(), &body.path, &body.data)
        })
        .map_err(Into::into)
        .map(|audit| AuditCreateResponse { data: audit })
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
    audit_id: &str,
) -> Result<AuditReadResponse, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::audit::read_by_id(data.driver(), service.as_ref(), &mut audit, audit_id)
        })
        .map_err(Into::into)
        .and_then(|audit| audit.ok_or_else(|| Error::NotFound))
        .map(|audit| AuditReadResponse { data: audit })
}
