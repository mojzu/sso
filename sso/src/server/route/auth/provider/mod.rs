mod local;

use crate::{
    api::{path as api_path, Api, AuthOauth2CallbackRequest},
    server::{
        route::{request_audit_meta, route_response_json},
        Data,
    },
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use futures::Future;

pub fn route_v1_scope() -> Scope {
    web::scope(api_path::PROVIDER)
        .service(local::route_v1_scope())
        .service(
            web::scope(api_path::GITHUB).service(
                web::resource(api_path::OAUTH2)
                    .route(web::get().to_async(github_oauth2_url_handler))
                    .route(web::post().to_async(github_oauth2_callback_handler)),
            ),
        )
        .service(
            web::scope(api_path::MICROSOFT).service(
                web::resource(api_path::OAUTH2)
                    .route(web::get().to_async(microsoft_oauth2_url_handler))
                    .route(web::post().to_async(microsoft_oauth2_callback_handler)),
            ),
        )
}

fn github_oauth2_url_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    request_audit_meta(&req)
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_github_oauth2_url(
                    data.driver(),
                    id,
                    audit_meta,
                    data.options().provider_github_oauth2_args(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn github_oauth2_callback_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthOauth2CallbackRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let request = body.into_inner();

    request_audit_meta(&req)
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_github_oauth2_callback(
                    data.driver(),
                    id,
                    audit_meta,
                    data.options().provider_github_oauth2_args(),
                    request,
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn microsoft_oauth2_url_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    request_audit_meta(&req)
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_microsoft_oauth2_url(
                    data.driver(),
                    id,
                    audit_meta,
                    data.options().provider_microsoft_oauth2_args(),
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}

fn microsoft_oauth2_callback_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
    body: web::Json<AuthOauth2CallbackRequest>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();
    let request = body.into_inner();

    request_audit_meta(&req)
        .and_then(move |audit_meta| {
            web::block(move || {
                Api::auth_provider_microsoft_oauth2_callback(
                    data.driver(),
                    id,
                    audit_meta,
                    data.options().provider_microsoft_oauth2_args(),
                    request,
                )
                .map_err(Into::into)
            })
            .map_err(Into::into)
        })
        .then(route_response_json)
}
