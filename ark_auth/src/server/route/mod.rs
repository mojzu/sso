pub mod auth;
pub mod key;
pub mod service;
pub mod user;

use crate::server::Error;
use actix_web::{web, HttpResponse, ResponseError};
use futures::{future, Future};
use serde::Serialize;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/v1")
        .service(
            web::resource("/ping")
                .data(route_json_config())
                .route(web::get().to(ping_handler)),
        )
        .service(auth::route_v1_scope())
        .service(key::route_v1_scope())
        .service(service::route_v1_scope())
        .service(user::route_v1_scope())
}

pub fn route_service(configuration: &mut web::ServiceConfig) {
    configuration.service(route_v1_scope());
}

fn ping_handler() -> actix_web::Result<HttpResponse> {
    let body = r#"pong"#;
    Ok(HttpResponse::Ok().json(body))
}

/// Route JSON size limit configuration.
pub fn route_json_config() -> web::JsonConfig {
    web::JsonConfig::default().limit(1024)
}

/// Route response empty handler.
pub fn route_response_empty<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(_res) => future::ok(HttpResponse::Ok().finish()),
        Err(err) => future::ok(err.error_response()),
    }
}

/// Route response handler.
pub fn route_response_json<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().json(res)),
        Err(err) => future::ok(err.error_response()),
    }
}
