//! # Authentication
pub mod key;
// pub mod oauth2;
pub mod reset;
pub mod token;

use crate::core;
use crate::server::{
    route_json_config, route_response_json, validate_key, validate_password, validate_token, Data,
    Error, ValidateFromValue,
};
use actix_web::{
    http::{header, StatusCode},
    middleware::identity::Identity,
    web, HttpResponse,
};
use futures::{future, Future};
use sha1::{Digest, Sha1};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordMeta {
    pub password_strength: Option<u8>,
    pub password_pwned: Option<bool>,
}

impl Default for PasswordMeta {
    fn default() -> Self {
        PasswordMeta {
            password_strength: None,
            password_pwned: None,
        }
    }
}

/// Returns password strength and pwned checks.
pub fn password_meta(
    data: &web::Data<Data>,
    password: Option<&str>,
) -> impl Future<Item = PasswordMeta, Error = Error> {
    match password {
        Some(password) => {
            let password_strength = password_meta_strength(password).then(|r| match r {
                Ok(entropy) => future::ok(Some(entropy.score)),
                Err(_e) => future::ok(None),
            });
            let password_pwned = password_meta_pwned(data, password).then(|r| match r {
                Ok(password_pwned) => future::ok(Some(password_pwned)),
                Err(_e) => future::ok(None),
            });
            future::Either::A(password_strength.join(password_pwned).map(
                |(password_strength, password_pwned)| PasswordMeta {
                    password_strength,
                    password_pwned,
                },
            ))
        }
        None => future::Either::B(future::ok(PasswordMeta::default())),
    }
}

/// Returns password strength test performed by `zxcvbn`.
/// <https://github.com/shssoichiro/zxcvbn-rs>
fn password_meta_strength(password: &str) -> impl Future<Item = zxcvbn::Entropy, Error = Error> {
    future::result(zxcvbn::zxcvbn(password, &[]).map_err(Error::Zxcvbn))
}

/// Returns true if password is present in `Pwned Passwords` index, else false.
/// <https://haveibeenpwned.com/Passwords>
fn password_meta_pwned(
    data: &web::Data<Data>,
    password: &str,
) -> impl Future<Item = bool, Error = Error> {
    let password_pwned_enabled = data.configuration().password_pwned_enabled();
    let user_agent = data.configuration().user_agent();

    if password_pwned_enabled {
        // Make request to API using first 5 characters of SHA1 password hash.
        let mut hash = Sha1::new();
        hash.input(password);
        let hash = format!("{:X}", hash.result());

        let client = actix_web::client::Client::new();
        let url = format!("https://api.pwnedpasswords.com/range/{:.5}", hash);
        future::Either::A(
            client
                .get(url)
                .header(header::USER_AGENT, user_agent)
                .send()
                .map_err(|_e| Error::ApiPwnedPasswords)
                // Receive OK response and return body as string.
                .and_then(|response| match response.status() {
                    StatusCode::OK => future::ok(response),
                    _ => future::err(Error::ApiPwnedPasswords),
                })
                .and_then(|mut response| {
                    response
                        .body()
                        .map_err(|_e| Error::ApiPwnedPasswords)
                        .and_then(|b| {
                            String::from_utf8(b.to_vec()).map_err(|_e| Error::ApiPwnedPasswords)
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
        future::Either::B(future::err(Error::ApiPwnedPasswords))
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

impl ValidateFromValue<LoginBody> for LoginBody {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub meta: PasswordMeta,
    pub data: core::UserToken,
}

/// API version 1 login route.
pub fn api_v1_login(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    LoginBody::from_value(body.into_inner())
        .and_then(|body| login_inner(data, id, body))
        .then(|result| route_response_json(result))
}

fn login_inner(
    data: web::Data<Data>,
    id: Option<String>,
    body: LoginBody,
) -> impl Future<Item = LoginResponse, Error = Error> {
    let (data1, body1) = (data.clone(), body.clone());

    web::block(move || {
        core::service_authenticate(data.driver(), id)
            .and_then(|service| {
                core::auth_login(data.driver(), &service, &body.email, &body.password)
            })
            // Map invalid password, not found errors to bad request to prevent leakage.
            // TODO(feature): Warning logs for bad requests.
            .map_err(|e| match e {
                // TODO(refactor): Refactor this.
                // DbError::InvalidPassword | DbError::NotFound => ApiError::BadRequest,
                _e => Error::Core(_e),
            })
    })
    .map_err(Into::into)
    .and_then(move |user_token| {
        let password_meta = password_meta(&data1, Some(&body1.password));
        let user_token = future::ok(user_token);
        password_meta.join(user_token)
    })
    .map(|(meta, user_token)| LoginResponse {
        meta,
        data: user_token,
    })
}

/// Version 1 API authentication scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/auth")
        .service(
            web::resource("/login")
                .route(web::post().data(route_json_config()).to_async(api_v1_login)),
        )
        // TODO(refactor): Refactor this.
        // .service(oauth2::api_v1_scope())
        .service(key::api_v1_scope())
        .service(reset::api_v1_scope())
        .service(token::api_v1_scope())
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    #[validate(custom = "validate_token")]
    pub token: String,
}

impl ValidateFromValue<TokenBody> for TokenBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub data: core::UserToken,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyBody {
    #[validate(custom = "validate_key")]
    pub key: String,
}

impl ValidateFromValue<KeyBody> for KeyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub data: core::UserKey,
}
