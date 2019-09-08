use crate::{
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    server_api::{path, AuthTokenBody, AuthTokenPartialResponse, AuthTokenResponse},
    AuditData, AuditMeta, Auth, Key, ServerResult, ServerValidateFromValue,
};
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
            web::block(move || {
                verify_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.token,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn verify_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    token: String,
    audit_data: Option<AuditData>,
) -> ServerResult<AuthTokenPartialResponse> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::token_verify(
                data.driver(),
                &service,
                &mut audit,
                &token,
                audit_data.as_ref(),
            )
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
            web::block(move || {
                refresh_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.token,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn refresh_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    token: String,
    audit_data: Option<AuditData>,
) -> ServerResult<AuthTokenResponse> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::token_refresh(
                data.driver(),
                &service,
                &mut audit,
                &token,
                audit_data.as_ref(),
                data.options().access_token_expires(),
                data.options().refresh_token_expires(),
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
            web::block(move || {
                revoke_inner(
                    data.get_ref(),
                    audit_meta,
                    id,
                    body.token,
                    body.audit.map(Into::into),
                )
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn revoke_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    token: String,
    audit_data: Option<AuditData>,
) -> ServerResult<usize> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::token_revoke(
                data.driver(),
                &service,
                &mut audit,
                &token,
                audit_data.as_ref(),
            )
        })
        .map_err(Into::into)
}
