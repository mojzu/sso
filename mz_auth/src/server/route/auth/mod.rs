mod key;
mod provider;
mod token;

use crate::{
    api_path,
    api_type::AuthTotpRequest,
    server::{
        route::{request_audit_meta, route_response_empty},
        Data,
    },
    Api,
};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse};
use futures::Future;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(api_path::AUTH)
        .service(provider::route_v1_scope())
        .service(key::route_v1_scope())
        .service(token::route_v1_scope())
        .service(web::resource(api_path::TOTP).route(web::post().to_async(totp_handler)))
}

fn totp_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthTotpRequest>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_totp(data.driver(), id, audit_meta, request).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}
