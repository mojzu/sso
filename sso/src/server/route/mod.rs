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
