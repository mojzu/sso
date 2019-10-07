use crate::{
    api_path,
    api_type::AuthTokenRequest,
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
    Api,
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;

pub fn route_v1_scope() -> Scope {
    web::scope(api_path::TOKEN)
        .service(web::resource(api_path::VERIFY).route(web::post().to_async(verify_handler)))
        .service(web::resource(api_path::REFRESH).route(web::post().to_async(refresh_handler)))
        .service(web::resource(api_path::REVOKE).route(web::post().to_async(revoke_handler)))
}

fn verify_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_token_verify(data.driver(), id, audit_meta, request).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn refresh_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_token_refresh(
                    data.driver(),
                    id,
                    audit_meta,
                    request,
                    data.options().access_token_expires(),
                    data.options().refresh_token_expires(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_token_revoke(data.driver(), id, audit_meta, request).map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_empty)
}
