use crate::core;
use crate::core::AuditMeta;
use crate::server::api::{path, AuthKeyBody, AuthKeyResponse};
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::KEY)
        .service(web::resource(path::VERIFY).route(web::post().to_async(verify_handler)))
        .service(web::resource(path::REVOKE).route(web::post().to_async(revoke_handler)))
}

fn verify_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthKeyBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || verify_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn verify_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthKeyBody,
) -> Result<AuthKeyResponse, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            // TODO(refactor): Implement custom audit logs.
            core::auth::key_verify(data.driver(), &service, &mut audit, &body.key) // &body.audit
        })
        .map_err(Into::into)
        .map(|user_key| AuthKeyResponse { data: user_key })
}

fn revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthKeyBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || revoke_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthKeyBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::key_revoke(data.driver(), &service, &mut audit, &body.key) // &body.audit
        })
        .map_err(Into::into)
}
