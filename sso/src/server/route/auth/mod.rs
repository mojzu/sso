mod key;
mod provider;
mod token;

use crate::{
    api::{self, ValidateRequestQuery},
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    Core,
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;

pub fn route_v1_scope() -> Scope {
    web::scope(api::path::AUTH)
        .service(provider::route_v1_scope())
        .service(key::route_v1_scope())
        .service(token::route_v1_scope())
        .service(web::resource(api::path::TOTP).route(web::post().to_async(totp_handler)))
        .service(
            web::resource(api::path::CSRF)
                .route(web::get().to_async(csrf_create_handler))
                .route(web::post().to_async(csrf_verify_handler)),
        )
}

fn totp_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTotpRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::auth_totp(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_empty)
}

fn csrf_create_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = api::AuthCsrfCreateRequest::from_str_fut(req.query_string());

    audit_meta
        .join(request)
        .and_then(move |(audit_meta, request)| {
            web::block(move || {
                api::auth_csrf_create(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                    Core::default_csrf_expires_s(),
                )
            })
            .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn csrf_verify_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthCsrfVerifyRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::auth_csrf_verify(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_empty)
}
