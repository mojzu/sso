mod audit;
mod auth;
mod key;
mod service;
mod user;

use crate::{
    api::{self, ApiError, ApiResult},
    server::Data,
    util::HeaderAuth,
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
        .service(web::resource(api::path::PING).route(web::get().to(ping_handler)))
        .service(web::resource(api::path::METRICS).route(web::get().to_async(metrics_handler)))
        .service(audit::route_v1_scope())
        .service(auth::route_v1_scope())
        .service(key::route_v1_scope())
        .service(service::route_v1_scope())
        .service(user::route_v1_scope())
}

fn ping_handler() -> Result<HttpResponse> {
    let body = api::server_ping();
    Ok(HttpResponse::Ok().json(body))
}

fn metrics_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let audit_meta = request_audit_meta(&req);
    let key_value = id.identity();

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || api::server_metrics(data.driver(), audit_meta, key_value))
                .map_err(Into::into)
        })
        .map_err(Into::into)
        .then(route_response_text)
}

/// Build audit meta from HTTP request.
fn request_audit_meta(req: &HttpRequest) -> future::FutureResult<AuditMeta, ApiError> {
    let connection_info = req.connection_info();
    let remote = connection_info
        .remote()
        .ok_or_else(|| ApiError::BadRequest(DriverError::HttpHeader));

    let user_agent = req
        .headers()
        .get(http::header::USER_AGENT)
        .ok_or_else(|| ApiError::BadRequest(DriverError::HttpHeader))
        .and_then(|x| {
            x.to_str()
                .map_err(|_err| ApiError::BadRequest(DriverError::HttpHeader))
        });

    let forwarded = req.headers().get(http::header::FORWARDED);
    let forwarded = if let Some(forwarded) = forwarded {
        forwarded
            .to_str()
            .map_err(|_err| ApiError::BadRequest(DriverError::HttpHeader))
            .map(|x| Some(x.to_owned()))
    } else {
        Ok(None)
    };

    let user = req.headers().get(HEADER_USER_AUTHORISATION_NAME);
    let user = if let Some(user) = user {
        user.to_str()
            .map_err(|_err| ApiError::BadRequest(DriverError::HttpHeader))
            .map(|x| HeaderAuth::parse(x))
    } else {
        Ok(None)
    };

    future::result(remote.and_then(|remote| {
        let user_agent = user_agent?;
        let forwarded = forwarded?;
        let user = user?;
        Ok(AuditMeta::new(user_agent, remote, forwarded, user))
    }))
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
