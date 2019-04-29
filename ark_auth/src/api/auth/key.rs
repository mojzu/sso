use crate::api::{
    auth::{KeyBody, KeyResponse},
    authenticate, body_json_config, ApiData, ApiError, FromJsonValue,
};
use crate::db::DbError;
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse, ResponseError};
use futures::{future, Future};

/// Version 1 authentication key routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/key")
        .service(
            web::resource("/verify")
                .route(web::post().data(body_json_config()).to_async(v1_key_verify)),
        )
        .service(
            web::resource("/revoke")
                .route(web::post().data(body_json_config()).to_async(v1_key_revoke)),
        )
}

pub fn v1_key_verify(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    KeyBody::from_value(body.into_inner())
        .and_then(move |body| key_verify_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn key_verify_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: KeyBody,
) -> impl Future<Item = KeyResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_key_verify(&body.key, &service)
                .map_err(map_bad_request)
        })
    })
    .map_err(Into::into)
    .map(|data| KeyResponse { data })
}

pub fn v1_key_revoke(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    KeyBody::from_value(body.into_inner())
        .and_then(move |body| key_revoke_inner(data, id, body))
        .then(|r| match r {
            Ok(_r) => future::ok(HttpResponse::Ok().finish()),
            Err(e) => future::ok(e.error_response()),
        })
}

fn key_revoke_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: KeyBody,
) -> impl Future<Item = (), Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_key_revoke(&body.key, &service)
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
