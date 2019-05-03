//! # Reset
use crate::core;
use crate::server::{
    auth::{password_meta, PasswordMeta},
    route_json_config, route_response_empty, route_response_json, validate_password,
    validate_token, Data, Error, ValidateFromValue,
};
use actix_web::{middleware::identity::Identity, web, HttpResponse};
use futures::Future;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordBody {
    #[validate(email)]
    email: String,
}

impl ValidateFromValue<ResetPasswordBody> for ResetPasswordBody {}

/// API version 1 password route.
pub fn api_v1_password(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ResetPasswordBody::from_value(body.into_inner())
        .and_then(|body| password_inner(data, id, body))
        .then(|result| route_response_empty(result))
}

fn password_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: ResetPasswordBody,
) -> impl Future<Item = usize, Error = Error> {
    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| core::auth_reset_password(data.driver(), &service, &body.email))
            .map_err(Into::into)
    })
    .map_err(Into::into)
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordConfirmBody {
    #[validate(custom = "validate_token")]
    token: String,
    #[validate(custom = "validate_password")]
    password: String,
}

impl ValidateFromValue<ResetPasswordConfirmBody> for ResetPasswordConfirmBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordConfirmResponse {
    pub meta: PasswordMeta,
}

/// API version 1 password confirm route.
pub fn api_v1_password_confirm(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    ResetPasswordConfirmBody::from_value(body.into_inner())
        .and_then(|body| password_confirm_inner(data, id, body))
        .then(|result| route_response_json(result))
}

fn password_confirm_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: ResetPasswordConfirmBody,
) -> impl Future<Item = ResetPasswordConfirmResponse, Error = Error> {
    let (data1, body1) = (data.clone(), body.clone());

    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| {
                core::auth_reset_password_confirm(
                    data.driver(),
                    &service,
                    &body.token,
                    &body.password,
                )
            })
            .map_err(Into::into)
    })
    .map_err(Into::into)
    .and_then(move |_| password_meta(&data1, Some(&body1.password)))
    .map(|meta| ResetPasswordConfirmResponse { meta })
}

/// Version 1 API authentication reset scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/reset")
        .service(
            web::resource("/password").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_password),
            ),
        )
        .service(
            web::resource("/password/confirm").route(
                web::post()
                    .data(route_json_config())
                    .to_async(api_v1_password_confirm),
            ),
        )
}
