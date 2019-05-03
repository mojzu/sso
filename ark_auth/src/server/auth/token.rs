//! # Token
use crate::core;
use crate::server::{
    auth::{TokenBody, TokenResponse},
    route_json_config, route_response_empty, route_response_json, Data, Error, ValidateFromValue,
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;

/// API version 1 verify route.
pub fn api_v1_verify(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| verify_inner(data, id, body))
        .then(|result| route_response_json(result))
}

fn verify_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: TokenBody,
) -> impl Future<Item = TokenResponse, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::auth_token_verify(data.driver(), &service, &body.token))
            .map_err(Into::into)
    })
    .map_err(Into::into)
    .map(|user_token| TokenResponse { data: user_token })
}

/// API version 1 refresh route.
pub fn api_v1_refresh(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| refresh_inner(data, id, body))
        .then(|result| route_response_json(result))
}

fn refresh_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: TokenBody,
) -> impl Future<Item = TokenResponse, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::auth_token_refresh(data.driver(), &service, &body.token))
            .map_err(Into::into)
    })
    .map_err(Into::into)
    .map(|user_token| TokenResponse { data: user_token })
}

/// API version 1 revoke route.
pub fn api_v1_revoke(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| revoke_inner(data, id, body))
        .then(|result| route_response_empty(result))
}

fn revoke_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: TokenBody,
) -> impl Future<Item = usize, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::auth_token_revoke(data.driver(), &service, &body.token))
            .map_err(Into::into)
    })
    .map_err(Into::into)
}

/// Version 1 API authentication token scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/token")
        .service(
            web::resource("/verify").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_verify),
            ),
        )
        .service(
            web::resource("/refresh").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_refresh),
            ),
        )
        .service(
            web::resource("/revoke").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_revoke),
            ),
        )
}
