//! # Key
use crate::server::{
    auth::{KeyBody, KeyResponse},
    route_json_config, route_response_empty, route_response_json, Data, Error, ValidateFromValue,
};

/// API version 1 verify route.
pub fn api_v1_verify(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    KeyBody::from_value(body.into_inner())
        .and_then(|body| verify_inner(data, id, body))
        .then(|result| route_response_json(result))
}

fn verify_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: KeyBody,
) -> impl Future<Item = KeyResponse, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::auth_key_verify(data.driver(), &service, &body.key))
            .map_err(Into::into)
    })
    .map_err(Into::into)
    .map(|user_key| KeyResponse { data: user_key })
}

/// API version 1 revoke route.
pub fn api_v1_revoke(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    KeyBody::from_value(body.into_inner())
        .and_then(|body| revoke_inner(data, id, body))
        .then(|result| route_response_empty(result))
}

fn revoke_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: KeyBody,
) -> impl Future<Item = usize, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::auth_key_revoke(data.driver(), &service, &body.key))
            .map_err(Into::into)
    })
    .map_err(Into::into)
}

/// Version 1 API authentication key scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/key")
        .service(
            web::resource("/verify").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_verify),
            ),
        )
        .service(
            web::resource("/revoke").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_key_revoke),
            ),
        )
}
