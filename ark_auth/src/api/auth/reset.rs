use crate::api::{
    auth::{
        check_password_meta, validate_password, validate_token, PasswordMetaResponse, TokenResponse,
    },
    authenticate, body_json_config, ApiData, ApiError, BodyFromValue,
};
use crate::db::DbError;
use crate::email;
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse, ResponseError};
use futures::{future, Future};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordBody {
    #[validate(email)]
    email: String,
}

impl BodyFromValue<ResetPasswordBody> for ResetPasswordBody {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ResetPasswordConfirmBody {
    #[validate(custom = "validate_token")]
    token: String,
    #[validate(custom = "validate_password")]
    password: String,
}

impl BodyFromValue<ResetPasswordConfirmBody> for ResetPasswordConfirmBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordConfirmResponse {
    meta: PasswordMetaResponse,
}

/// Version 1 authentication reset routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/reset")
        .service(
            web::resource("/password").route(
                web::post()
                    .data(body_json_config())
                    .to_async(v1_reset_password),
            ),
        )
        .service(
            web::resource("/password/confirm").route(
                web::post()
                    .data(body_json_config())
                    .to_async(v1_reset_password_confirm),
            ),
        )
}

pub fn v1_reset_password(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    ResetPasswordBody::from_value(body.into_inner())
        .and_then(move |body| reset_password_inner(data, id, body))
        .then(|r| match r {
            Ok(_r) => future::ok(HttpResponse::Ok().finish()),
            Err(e) => future::ok(e.error_response()),
        })
}

fn reset_password_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: ResetPasswordBody,
) -> impl Future<Item = TokenResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id)
            .and_then(|service| {
                let reset = data
                    .db
                    .auth_reset_password(&body.email, &service)
                    .map_err(map_bad_request)?;
                Ok((service, reset))
            })
            .and_then(|(service, (user, token_response))| {
                // Send user email with reset password confirmation link.
                match email::send_reset_password(
                    data.smtp(),
                    &user,
                    &service,
                    &token_response.token,
                ) {
                    Ok(_) => Ok(token_response),
                    // Log warning in case of failure to send email.
                    Err(e) => {
                        warn!("Failed to send reset password email ({})", e);
                        Ok(token_response)
                    }
                }
            })
    })
    .map_err(Into::into)
    .map(|data| TokenResponse { data })
}

pub fn v1_reset_password_confirm(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    ResetPasswordConfirmBody::from_value(body.into_inner())
        .and_then(move |body| reset_password_confirm_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn reset_password_confirm_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: ResetPasswordConfirmBody,
) -> impl Future<Item = ResetPasswordConfirmResponse, Error = ApiError> {
    let (data1, body1) = (data.clone(), body.clone());

    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_reset_password_confirm(&body.token, &body.password, &service)
                .map_err(map_bad_request)
        })
    })
    .map_err(Into::into)
    .and_then(move |_| check_password_meta(&data1, &body1.password))
    .map(|meta| ResetPasswordConfirmResponse { meta })
}

/// Map not found errors to bad request to prevent leakage.
fn map_bad_request(e: DbError) -> ApiError {
    match e {
        DbError::InvalidPasswordRevision | DbError::NotFound => ApiError::BadRequest,
        _e => ApiError::Db(_e),
    }
}
