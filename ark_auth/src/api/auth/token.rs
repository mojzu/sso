use crate::api::{
    auth::{TokenBody, TokenResponse},
    authenticate, body_json_config, ApiData, ApiError, BodyFromValue,
};
use crate::db::DbError;
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse, ResponseError};
use futures::{future, Future};

/// Version 1 authentication token routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/token")
        .service(
            web::resource("/verify").route(
                web::post()
                    .data(body_json_config())
                    .to_async(v1_token_verify),
            ),
        )
        .service(
            web::resource("/refresh").route(
                web::post()
                    .data(body_json_config())
                    .to_async(v1_token_refresh),
            ),
        )
        .service(
            web::resource("/revoke").route(
                web::post()
                    .data(body_json_config())
                    .to_async(v1_token_revoke),
            ),
        )
}

pub fn v1_token_verify(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(move |body| token_verify_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn token_verify_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: TokenBody,
) -> impl Future<Item = TokenResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_token_verify(&body.token, &service)
                .map_err(map_bad_request)
        })
    })
    .map_err(Into::into)
}

pub fn v1_token_refresh(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(move |body| token_refresh_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn token_refresh_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: TokenBody,
) -> impl Future<Item = TokenResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_token_refresh(&body.token, &service)
                .map_err(map_bad_request)
        })
    })
    .map_err(Into::into)
}

pub fn v1_token_revoke(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    TokenBody::from_value(body.into_inner())
        .and_then(move |body| token_revoke_inner(data, id, body))
        .then(|r| match r {
            Ok(_r) => future::ok(HttpResponse::Ok().finish()),
            Err(e) => future::ok(e.error_response()),
        })
}

fn token_revoke_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: TokenBody,
) -> impl Future<Item = (), Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_token_revoke(&body.token, &service)
                .map_err(map_bad_request)
        })
    })
    .map_err(Into::into)
}

/// Map not found errors to bad request to prevent leakage.
fn map_bad_request(e: DbError) -> ApiError {
    match e {
        DbError::NotFound => ApiError::BadRequest,
        _e => ApiError::Db(_e),
    }
}
