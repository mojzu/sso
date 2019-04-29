pub mod key;
pub mod oauth2;
pub mod reset;
pub mod token;

use crate::api::{authenticate, body_json_config, ApiData, ApiError, FromJsonValue};
use crate::db::{DbError, KeyData, TokenData};
use actix_web::http::{header, StatusCode};
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
        .service(oauth2::v1_service())
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

pub fn validate_id(id: i64) -> Result<(), ValidationError> {
    if id < 1 {
        Err(ValidationError::new("invalid_id"))
    } else {
        Ok(())
    }
}

pub fn validate_unsigned(id: i64) -> Result<(), ValidationError> {
    if id < 0 {
        Err(ValidationError::new("invalid_unsigned"))
    } else {
        Ok(())
    }
}

pub fn validate_order(order: &str) -> Result<(), ValidationError> {
    match order {
        "asc" | "desc" => Ok(()),
        _ => Err(ValidationError::new("invalid_order")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate_password")]
    pub password: String,
}

impl FromJsonValue<LoginBody> for LoginBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordMetaResponse {
    pub password_strength: Option<u8>,
    pub password_pwned: Option<bool>,
}

impl Default for PasswordMetaResponse {
    fn default() -> Self {
        PasswordMetaResponse {
            password_strength: None,
            password_pwned: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub meta: PasswordMetaResponse,
    pub data: TokenData,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    #[validate(custom = "validate_token")]
    pub token: String,
}

impl FromJsonValue<TokenBody> for TokenBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub data: TokenData,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyBody {
    #[validate(custom = "validate_key")]
    pub key: String,
}

impl FromJsonValue<KeyBody> for KeyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub data: KeyData,
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
) -> impl Future<Item = LoginResponse, Error = ApiError> {
    let (data1, data2, body1, body2) = (data.clone(), data.clone(), body.clone(), body.clone());

    web::block(move || authenticate(&data, id))
        .map_err(Into::into)
        .and_then(move |service| {
            web::block(move || {
                data1
                    .db
                    .auth_login(&body1.email, &body1.password, &service)
                    // Map invalid password, not found errors to bad request to prevent leakage.
                    // TODO(feature): Warning logs for bad requests.
                    .map_err(|e| match e {
                        DbError::InvalidPassword | DbError::NotFound => ApiError::BadRequest,
                        _e => ApiError::Db(_e),
                    })
            })
            .map_err(Into::into)
            .and_then(move |token_response| {
                let password_meta = check_password_meta(&data2, &body2.password);
                let token_response = future::ok(token_response);
                password_meta.join(token_response)
            })
        })
        .map(|(meta, data)| LoginResponse { meta, data })
}

/// Returns password strength and pwned checks.
pub fn check_password_meta(
    data: &web::Data<ApiData>,
    password: &str,
) -> impl Future<Item = PasswordMetaResponse, Error = ApiError> {
    let password_strength = check_password_strength(password).then(|r| match r {
        Ok(entropy) => future::ok(Some(entropy.score)),
        Err(_e) => future::ok(None),
    });
    let password_pwned = check_password_pwned(data, password).then(|r| match r {
        Ok(password_pwned) => future::ok(Some(password_pwned)),
        Err(_e) => future::ok(None),
    });
    password_strength
        .join(password_pwned)
        .map(|(password_strength, password_pwned)| PasswordMetaResponse {
            password_strength,
            password_pwned,
        })
}

/// Returns password strength test performed by `zxcvbn`.
/// <https://github.com/shssoichiro/zxcvbn-rs>
fn check_password_strength(
    password: &str,
) -> impl Future<Item = zxcvbn::Entropy, Error = ApiError> {
    future::result(zxcvbn::zxcvbn(password, &[]).map_err(|_e| ApiError::Unwrap("zxcvbn failed")))
}

/// Returns true if password is present in `Pwned Passwords` index, else false.
/// <https://haveibeenpwned.com/Passwords>
fn check_password_pwned(
    data: &web::Data<ApiData>,
    password: &str,
) -> impl Future<Item = bool, Error = ApiError> {
    use sha1::{Digest, Sha1};

    if data.password_pwned() {
        // Make request to API using first 5 characters of SHA1 password hash.
        let mut hash = Sha1::new();
        hash.input(password);
        let hash = format!("{:X}", hash.result());

        let client = actix_web::client::Client::new();
        let url = format!("https://api.pwnedpasswords.com/range/{:.5}", hash);
        future::Either::A(
            client
                .get(url)
                .header(header::USER_AGENT, data.user_agent())
                .send()
                .map_err(|_e| ApiError::Unwrap("failed to client.request"))
                // Receive OK response and return body as string.
                .and_then(|response| match response.status() {
                    StatusCode::OK => future::ok(response),
                    _ => future::err(ApiError::Unwrap("failed to receive ok response")),
                })
                .and_then(|mut response| {
                    response
                        .body()
                        .map_err(|_e| ApiError::Unwrap("failed to parse text"))
                        .and_then(|b| {
                            String::from_utf8(b.to_vec())
                                .map_err(|_e| ApiError::Unwrap("failed to parse text"))
                        })
                })
                // Compare suffix of hash to lines to determine if password is pwned.
                .and_then(move |text| {
                    for line in text.lines() {
                        if hash[5..] == line[..35] {
                            return Ok(true);
                        }
                    }
                    Ok(false)
                }),
        )
    } else {
        future::Either::B(future::err(ApiError::Unwrap(
            "password pwned check disabled",
        )))
    }
}
