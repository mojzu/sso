pub mod audit;
pub mod auth;
pub mod key;
pub mod service;
pub mod user;

use crate::core;
use crate::core::AuditMeta;
use crate::server::api::path;
use crate::server::{Data, Error};
use actix_identity::Identity;
use actix_web::{web, HttpRequest, HttpResponse, ResponseError};
use futures::{future, Future};
use serde::Serialize;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::V1)
        .service(web::resource(path::PING).route(web::get().to(ping_handler)))
        .service(web::resource(path::METRICS).route(web::get().to_async(metrics_handler)))
        .service(audit::route_v1_scope())
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

fn metrics_handler(
    data: web::Data<Data>,
    req: HttpRequest,
    id: Identity,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();
    let audit_meta = request_audit_meta(&req);

    audit_meta
        .and_then(|audit_meta| {
            web::block(move || metrics_inner(data.get_ref(), audit_meta, id)).map_err(Into::into)
        })
        .then(route_response_text)
}

fn metrics_inner(data: &Data, audit_meta: AuditMeta, id: Option<String>) -> Result<String, Error> {
    core::key::authenticate(data.driver(), audit_meta, id)
        .and_then(|(service, mut audit)| {
            core::metrics::read(data.driver(), service.as_ref(), &mut audit, data.registry())
        })
        .map_err(Into::into)
}

/// Build audit meta from HTTP request.
pub fn request_audit_meta(req: &HttpRequest) -> future::FutureResult<AuditMeta, Error> {
    let connection_info = req.connection_info();
    let remote = connection_info.remote().ok_or_else(|| Error::BadRequest);

    let user_agent = req
        .headers()
        .get(http::header::USER_AGENT)
        .ok_or_else(|| Error::BadRequest)
        .and_then(|x| x.to_str().map_err(|_err| Error::BadRequest));

    let forwarded = req.headers().get(http::header::FORWARDED);
    let forwarded = if let Some(forwarded) = forwarded {
        forwarded
            .to_str()
            .map_err(|_err| Error::BadRequest)
            .map(|x| Some(x.to_owned()))
    } else {
        Ok(None)
    };

    future::result(remote.and_then(|remote| {
        let user_agent = user_agent?;
        let forwarded = forwarded?;
        Ok(AuditMeta::new(user_agent, remote, forwarded))
    }))
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

/// Route response JSON handler.
pub fn route_response_json<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().json(res)),
        Err(err) => future::ok(err.error_response()),
    }
}

/// Route response text handler.
pub fn route_response_text(
    result: Result<String, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().body(res)),
        Err(err) => future::ok(err.error_response()),
    }
}
