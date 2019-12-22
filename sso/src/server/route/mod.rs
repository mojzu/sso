mod audit;
mod auth;
mod key;
mod service;
mod user;

use crate::{
    api::{self, ApiError, ApiResult},
    pattern::HeaderAuth,
    server::Data,
    AuditMeta, DriverError, HEADER_USER_AUTHORISATION_NAME,
};
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, ResponseError, Result, Scope};
use futures::{future, Future};
use serde::Serialize;

pub fn route_service(config: &mut web::ServiceConfig) {
    config.service(route_v1_scope());
}

fn route_v1_scope() -> Scope {
    web::scope(api::path::V1)
        .service(audit::route_v1_scope())
        .service(auth::route_v1_scope())
        .service(key::route_v1_scope())
        .service(service::route_v1_scope())
        .service(user::route_v1_scope())
}

/// Route response empty handler.
fn route_response_empty<T: Serialize>(
    result: ApiResult<T>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    match result {
        Ok(_res) => future::ok(HttpResponse::Ok().finish()),
        Err(err) => future::ok(err.error_response()),
    }
}

/// Route response JSON handler.
fn route_response_json<T: Serialize>(
    result: ApiResult<T>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().json(res)),
        Err(err) => future::ok(err.error_response()),
    }
}

/// Route response text handler.
fn route_response_text(
    result: ApiResult<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().body(res)),
        Err(err) => future::ok(err.error_response()),
    }
}
