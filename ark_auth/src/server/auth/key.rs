use crate::core;
use crate::server::{
    auth::{KeyBody, KeyResponse},
    route_json_config, route_response_empty, route_response_json, Data, Error, ValidateFromValue,
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;

fn verify_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    KeyBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || verify_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_json)
}

fn verify_inner(data: &Data, id: Option<String>, body: &KeyBody) -> Result<KeyResponse, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| core::auth::key_verify(data.driver(), &service, &body.key))
        .map_err(Into::into)
        .map(|user_key| KeyResponse { data: user_key })
}

fn revoke_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    KeyBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || revoke_inner(data.get_ref(), id, &body)).map_err(Into::into)
        })
        .then(route_response_empty)
}

fn revoke_inner(data: &Data, id: Option<String>, body: &KeyBody) -> Result<usize, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| core::auth::key_revoke(data.driver(), &service, &body.key))
        .map_err(Into::into)
}

/// Version 1 API authentication key scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/key")
        .service(
            web::resource("/verify")
                .data(route_json_config())
                .route(web::post().to_async(verify_handler)),
        )
        .service(
            web::resource("/revoke")
                .data(route_json_config())
                .route(web::post().to_async(revoke_handler)),
        )
}
