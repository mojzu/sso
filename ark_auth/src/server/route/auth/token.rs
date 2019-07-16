use crate::core;
use crate::core::AuditMeta;
use crate::server::api::{path, AuthTokenBody, AuthTokenPartialResponse, AuthTokenResponse};
use crate::server::route::{request_audit_meta, route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::TOKEN)
        .service(web::resource(path::VERIFY).route(web::post().to_async(verify_handler)))
        .service(web::resource(path::REFRESH).route(web::post().to_async(refresh_handler)))
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
    let body = AuthTokenBody::from_value(body.into_inner());

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
    body: &AuthTokenBody,
) -> Result<AuthTokenPartialResponse, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::token_verify(data.driver(), &service, &mut audit, &body.token)
        })
        .map_err(Into::into)
        .map(|user_token| AuthTokenPartialResponse { data: user_token })
}

fn refresh_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthTokenBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || refresh_inner(data.get_ref(), audit_meta, id, &body))
                .map_err(Into::into)
        })
        .then(route_response_json)
}

fn refresh_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    body: &AuthTokenBody,
) -> Result<AuthTokenResponse, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::token_refresh(
                data.driver(),
                &service,
                &mut audit,
                &body.token,
                data.configuration().core_access_token_expires(),
                data.configuration().core_refresh_token_expires(),
            )
        })
        .map_err(Into::into)
        .map(|user_token| AuthTokenResponse { data: user_token })
}

fn revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthTokenBody::from_value(body.into_inner());

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
    body: &AuthTokenBody,
) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::auth::token_revoke(data.driver(), &service, &mut audit, &body.token)
        })
        .map_err(Into::into)
}
