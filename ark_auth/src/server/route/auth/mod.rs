pub mod key;
pub mod oauth2;
pub mod reset;
pub mod token;

use crate::{
    core,
    server::{route_json_config, route_response_json, validate, Data, Error, FromJsonValue},
};
use actix_web::{
    http::{header, StatusCode},
    middleware::identity::Identity,
    web, HttpResponse,
};
use futures::{future, Future};
use sha1::{Digest, Sha1};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
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
    data: &Data,
    password: Option<&str>,
) -> impl Future<Item = PasswordMeta, Error = Error> {
    match password {
        Some(password) => {
            let password_strength = password_meta_strength(password).then(|r| match r {
                Ok(entropy) => future::ok(Some(entropy.score)),
                Err(err) => {
                    warn!("{}", err);
                    future::ok(None)
                }
            });
            let password_pwned = password_meta_pwned(data, password).then(|r| match r {
                Ok(password_pwned) => future::ok(Some(password_pwned)),
                Err(err) => {
                    warn!("{}", err);
                    future::ok(None)
                }
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
fn password_meta_pwned(data: &Data, password: &str) -> impl Future<Item = bool, Error = Error> {
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
                // TODO(test): Rustls support broken? Improve error messages.
                .map_err(|_err| Error::ApiPwnedPasswords)
                // Receive OK response and return body as string.
                .and_then(|response| match response.status() {
                    StatusCode::OK => future::ok(response),
                    _ => future::err(Error::ApiPwnedPasswords),
                })
                .and_then(|mut response| {
                    response
                        .body()
                        .map_err(|_err| Error::ApiPwnedPasswords)
                        .and_then(|b| {
                            String::from_utf8(b.to_vec()).map_err(|_err| Error::ApiPwnedPasswords)
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

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct LoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::password")]
    pub password: String,
}

impl validate::FromJsonValue<LoginBody> for LoginBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub meta: PasswordMeta,
    pub data: core::UserToken,
}

fn login_handler(
    data: web::Data<Data>,
    id: Identity,
    body: web::Json<serde_json::Value>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let id = id.identity();

    LoginBody::from_value(body.into_inner())
        .and_then(|body| {
            web::block(move || {
                let user_token = login_inner(data.get_ref(), id, &body)?;
                Ok((data, body, user_token))
            })
            .map_err(Into::into)
        })
        .and_then(|(data, body, user_token)| {
            let password_meta = password_meta(data.get_ref(), Some(&body.password));
            let user_token = future::ok(user_token);
            password_meta.join(user_token)
        })
        .map(|(meta, user_token)| LoginResponse {
            meta,
            data: user_token,
        })
        .then(route_response_json)
}

fn login_inner(
    data: &Data,
    id: Option<String>,
    body: &LoginBody,
) -> Result<core::UserToken, Error> {
    core::key::authenticate_service(data.driver(), id)
        .and_then(|service| core::auth::login(data.driver(), &service, &body.email, &body.password))
        .map_err(Into::into)
}

/// Version 1 API authentication scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/auth")
        .service(
            web::resource("/login")
                .data(route_json_config())
                .route(web::post().to_async(login_handler)),
        )
        .service(oauth2::api_v1_scope())
        .service(key::api_v1_scope())
        .service(reset::api_v1_scope())
        .service(token::api_v1_scope())
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    #[validate(custom = "validate::token")]
    pub token: String,
}

impl validate::FromJsonValue<TokenBody> for TokenBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub data: core::UserToken,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyBody {
    #[validate(custom = "validate::key")]
    pub key: String,
}

impl validate::FromJsonValue<KeyBody> for KeyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub data: core::UserKey,
}
