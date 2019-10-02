mod key;
mod provider;
mod token;

use crate::{
    server::{
        route::{request_audit_meta, route_response_empty},
        Data,
    },
    server_api::{path, AuthTotpBody},
    AuditMeta, Auth, Key, ServerResult, ServerValidateFromValue,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;
use serde_json::Value;
use uuid::Uuid;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::AUTH)
        .service(provider::route_v1_scope())
        .service(key::route_v1_scope())
        .service(token::route_v1_scope())
        .service(web::resource(path::TOTP).route(web::post().to_async(totp_handler)))
}

fn totp_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let body = AuthTotpBody::from_value(body.into_inner());

    audit_meta
        .join(body)
        .and_then(|(audit_meta, body)| {
            web::block(move || totp_inner(data.get_ref(), audit_meta, id, body.user_id, body.totp))
                .map_err(Into::into)
        })
        .then(route_response_empty)
}

fn totp_inner(
    data: &Data,
    audit_meta: AuditMeta,
    id: Option<String>,
    user_id: Uuid,
    totp: String,
) -> ServerResult<()> {
    Key::authenticate_service(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            Auth::totp(data.driver(), &service, &mut audit, user_id, totp)
        })
        .map_err(Into::into)
}
