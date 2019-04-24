pub mod key;
pub mod oauth;
pub mod reset;
pub mod token;

use crate::api::{authenticate, body_json_config, ApiData, ApiError, BodyFromValue};
use crate::db::DbError;
use actix_web::{middleware::identity::Identity, web, Error, HttpResponse, ResponseError};
use futures::{future, Future};
use validator::{Validate, ValidationError};

/// Version 1 authentication routes scope.
pub fn v1_service() -> actix_web::Scope {
    web::scope("/auth")
        .service(
            web::resource("/login").route(web::post().data(body_json_config()).to_async(v1_login)),
        )
        .service(reset::v1_service())
        .service(token::v1_service())
        .service(key::v1_service())
        .service(oauth::v1_service())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.is_empty() || password.len() > 100 {
        Err(ValidationError::new("invalid_password"))
    } else {
        Ok(())
    }
}

pub fn validate_token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() || token.len() > 512 {
        Err(ValidationError::new("invalid_token"))
    } else {
        Ok(())
    }
}

pub fn validate_key(key: &str) -> Result<(), ValidationError> {
    if key.is_empty() || key.len() > 32 {
        Err(ValidationError::new("invalid_key"))
    } else {
        Ok(())
    }
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 100 {
        Err(ValidationError::new("invalid_name"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate_password")]
    pub password: String,
}

impl BodyFromValue<LoginBody> for LoginBody {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    #[validate(custom = "validate_token")]
    pub token: String,
}

impl BodyFromValue<TokenBody> for TokenBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub user_id: i64,
    pub token: String,
    pub token_expires: usize,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyBody {
    #[validate(custom = "validate_key")]
    pub key: String,
}

impl BodyFromValue<KeyBody> for KeyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub user_id: i64,
    pub key: String,
}

pub fn v1_login(
    data: web::Data<ApiData>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let id = id.identity();

    LoginBody::from_value(body.into_inner())
        .and_then(move |body| login_inner(data, id, body))
        .then(|r| match r {
            Ok(r) => future::ok(HttpResponse::Ok().json(r)),
            Err(e) => future::ok(e.error_response()),
        })
}

fn login_inner(
    data: web::Data<ApiData>,
    id: Option<String>,
    body: LoginBody,
) -> impl Future<Item = TokenResponse, Error = ApiError> {
    web::block(move || {
        authenticate(&data, id).and_then(|service| {
            data.db
                .auth_login(&body.email, &body.password, &service)
                // Map invalid password, not found errors to bad request to prevent leakage.
                // TODO(feature): Warning logs for bad requests.
                .map_err(|e| match e {
                    DbError::InvalidPassword | DbError::NotFound => ApiError::BadRequest,
                    _e => ApiError::Db(_e),
                })
        })
    })
    .map_err(Into::into)
}
