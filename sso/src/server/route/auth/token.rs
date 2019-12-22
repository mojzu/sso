use crate::{
    api,
    server::{
        route::{request_audit_meta, route_response_empty, route_response_json},
        Data,
    },
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;

fn verify_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::auth_token_verify(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn refresh_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || {
                api::auth_token_refresh(
                    data.driver(),
                    audit_meta,
                    id,
                    request,
                    data.options().access_token_expires(),
                    data.options().refresh_token_expires(),
                )
            })
            .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_json)
}

fn revoke_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<api::AuthTokenRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let id = id.identity();
    let request = body.into_inner();

    audit_meta
        .and_then(move |audit_meta| {
            web::block(move || api::auth_token_revoke(data.driver(), audit_meta, id, request))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_empty)
}
