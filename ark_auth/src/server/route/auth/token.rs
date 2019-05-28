use crate::{
    core,
    server::{
        route::auth::{TokenBody, TokenResponse},
        route_json_config, route_response_empty, route_response_json, Data, Error, FromJsonValue,
    },
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;

fn verify_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || verify_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn verify_inner(data: &Data, id: Option<String>, body: &TokenBody) -> Result<TokenResponse, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| core::auth::token_verify(data.driver(), &service, &body.token))
        .map_err(Into::into)
        .map(|user_token| TokenResponse { data: user_token })
}

// TODO(feature): Refresh counter and configurable limit, or separate refresh tokens.

fn refresh_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || refresh_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn refresh_inner(
    data: &Data,
    id: Option<String>,
    body: &TokenBody,
) -> Result<TokenResponse, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| {
            core::auth::token_refresh(
                data.driver(),
                &service,
                &body.token,
                data.configuration().token_exp(),
            )
        })
        .map_err(Into::into)
        .map(|user_token| TokenResponse { data: user_token })
}

fn revoke_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || revoke_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_empty)
}

fn revoke_inner(data: &Data, id: Option<String>, body: &TokenBody) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| core::auth::token_revoke(data.driver(), &service, &body.token))
        .map_err(Into::into)
}

/// Version 1 API authentication token scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/token")
        .service(
            web::resource("/verify")
                .data(route_json_config())
                .route(web::post().to_async(verify_handler)),
        )
        .service(
            web::resource("/refresh")
                .data(route_json_config())
                .route(web::post().to_async(refresh_handler)),
        )
        .service(
            web::resource("/revoke")
                .data(route_json_config())
                .route(web::post().to_async(revoke_handler)),
        )
}
