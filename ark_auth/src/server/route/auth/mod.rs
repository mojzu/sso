pub mod key;
pub mod provider;
pub mod token;

use crate::client::Get;
use crate::server::api::{path, AuthPasswordMeta};
use crate::server::{Data, Error, PwnedPasswordsError};
use actix_web::web;
use futures::{future, Future};
use sha1::{Digest, Sha1};

pub fn route_v1_scope() -> actix_web::Scope {
    web::scope(path::AUTH)
        .service(provider::route_v1_scope())
        .service(key::route_v1_scope())
        .service(token::route_v1_scope())
}

/// Returns password strength and pwned checks.
pub fn password_meta(
    data: &Data,
    password: Option<&str>,
) -> impl Future<Item = AuthPasswordMeta, Error = Error> {
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
                |(password_strength, password_pwned)| AuthPasswordMeta {
                    password_strength,
                    password_pwned,
                },
            ))
        }
        None => future::Either::B(future::ok(AuthPasswordMeta::default())),
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
    let password_pwned_enabled = data.options().password_pwned_enabled();

    if password_pwned_enabled {
        // Make request to API using first 5 characters of SHA1 password hash.
        let mut hash = Sha1::new();
        hash.input(password);
        let hash = format!("{:X}", hash.result());
        let route = format!("/range/{:.5}", hash);

        future::Either::A(
            // Make API request.
            data.client()
                .send(Get::text("https://api.pwnedpasswords.com", route))
                .map_err(|_err| unimplemented!())
                .and_then(|res| res.map_err(|_err| unimplemented!()))
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
