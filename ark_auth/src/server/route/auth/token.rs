use crate::core;
use crate::server::route::auth::{TokenBody, TokenPartialResponse, TokenResponse};
use crate::server::route::{route_response_empty, route_response_json};
use crate::server::{Data, Error, FromJsonValue};
use actix_identity::Identity;
use actix_web::{web, HttpResponse};
use futures::Future;
use serde_json::Value;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/token")
        .service(web::resource("/verify").route(web::post().to_async(verify_handler)))
        .service(web::resource("/refresh").route(web::post().to_async(refresh_handler)))
        .service(web::resource("/revoke").route(web::post().to_async(revoke_handler)))
}

fn verify_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || verify_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn verify_inner(
    data: &Data,
    id: Option<String>,
    body: &TokenBody,
) -> Result<TokenPartialResponse, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|(service, _)| core::auth::token_verify(data.driver(), &service, &body.token))
        .map_err(Into::into)
        .map(|user_token| TokenPartialResponse { data: user_token })
}

fn refresh_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
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
        .and_then(|(service, _)| {
            core::auth::token_refresh(
                data.driver(),
                &service,
                &body.token,
                data.configuration().core_access_token_expires(),
                data.configuration().core_refresh_token_expires(),
            )
        })
        .map_err(Into::into)
        .map(|user_token| TokenResponse { data: user_token })
}

fn revoke_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<Value>,
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
        .and_then(|(service, _)| core::auth::token_revoke(data.driver(), &service, &body.token))
        .map_err(Into::into)
}
