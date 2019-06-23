pub mod key;
pub mod provider;
pub mod token;

use crate::core;
use crate::server::{validate, Data, Error, FromJsonValue, PwnedPasswordsError};
use actix_web::http::{header, StatusCode};
use actix_web::web;
use futures::{future, Future};
use sha1::{Digest, Sha1};
use validator::Validate;

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope("/auth")
        .service(provider::route_v1_scope())
        .service(key::route_v1_scope())
        .service(token::route_v1_scope())
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    #[validate(custom = "validate::token")]
    pub token: String,
}

impl FromJsonValue<TokenBody> for TokenBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub data: core::UserToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPartialResponse {
    pub data: core::UserTokenPartial,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyBody {
    #[validate(custom = "validate::key")]
    pub key: String,
}

impl FromJsonValue<KeyBody> for KeyBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyResponse {
    pub data: core::UserKey,
}

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
                .map_err(|_err| Error::PwnedPasswords(PwnedPasswordsError::ActixClientSendRequest))
                // Receive OK response and return body as string.
                .and_then(|response| {
                    let status = response.status();
                    match status {
                        StatusCode::OK => future::ok(response),
                        _ => future::err(Error::PwnedPasswords(PwnedPasswordsError::StatusCode(
                            status,
                        ))),
                    }
                })
                .and_then(|mut response| {
                    response
                        .body()
                        .map_err(|_err| Error::PwnedPasswords(PwnedPasswordsError::ActixPayload))
                        .and_then(|b| {
                            String::from_utf8(b.to_vec()).map_err(|err| {
                                Error::PwnedPasswords(PwnedPasswordsError::FromUtf8(err))
                            })
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
        future::Either::B(future::err(Error::PwnedPasswords(
            PwnedPasswordsError::Disabled,
        )))
    }
}
