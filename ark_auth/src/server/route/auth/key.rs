use crate::core;
use crate::core::audit::AuditMeta;
use crate::server::route::auth::{KeyBody, KeyResponse};
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/key")
        .service(web::resource("/verify").route(web::post().to_async(verify_handler)))
        .service(web::resource("/revoke").route(web::post().to_async(revoke_handler)))
}

fn verify_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = KeyBody::from_value(body.into_inner());

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
    body: &KeyBody,
) -> Result<KeyResponse, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, _)| core::auth::key_verify(data.driver(), &service, &body.key))
        .map_err(Into::into)
        .map(|user_key| KeyResponse { data: user_key })
}

fn revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = KeyBody::from_value(body.into_inner());

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
    body: &KeyBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, _)| core::auth::key_revoke(data.driver(), &service, &body.key))
        .map_err(Into::into)
}
