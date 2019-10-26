use crate::{client_msg::Get, ClientActor, DriverError, UserPasswordMeta};
use actix::Addr;
use futures::{future, Future};
use sha1::{Digest, Sha1};

/// Returns password strength and pwned checks.
pub fn password_meta(
    enabled: bool,
    client: &Addr<ClientActor>,
    password: Option<&str>,
) -> impl Future<Item = UserPasswordMeta, Error = DriverError> {
    match password {
        Some(password) => {
            let password_strength = password_meta_strength(password).then(|r| match r {
                Ok(entropy) => future::ok(Some(entropy.score)),
                Err(err) => {
                    warn!("{}", err);
                    future::ok(None)
                }
            });
            let password_pwned = password_meta_pwned(enabled, client, password).then(|r| match r {
                Ok(password_pwned) => future::ok(Some(password_pwned)),
                Err(err) => {
                    warn!("{}", err);
                    future::ok(None)
                }
            });
            future::Either::A(password_strength.join(password_pwned).map(
                |(password_strength, password_pwned)| UserPasswordMeta {
                    password_strength,
                    password_pwned,
                },
            ))
        }
        None => future::Either::B(future::ok(UserPasswordMeta::default())),
    }
}

/// Returns password strength test performed by `zxcvbn`.
/// <https://github.com/shssoichiro/zxcvbn-rs>
fn password_meta_strength(
    password: &str,
) -> impl Future<Item = zxcvbn::Entropy, Error = DriverError> {
    // TODO(fix): Fix "Zxcvbn cannot evaluate a blank password" warning.
    future::result(zxcvbn::zxcvbn(password, &[]).map_err(DriverError::Zxcvbn))
}

/// Returns true if password is present in `Pwned Passwords` index, else false.
/// <https://haveibeenpwned.com/Passwords>
fn password_meta_pwned(
    enabled: bool,
    client: &Addr<ClientActor>,
    password: &str,
) -> impl Future<Item = bool, Error = DriverError> {
    if enabled {
        // Make request to API using first 5 characters of SHA1 password hash.
        let mut hash = Sha1::new();
        hash.input(password);
        let hash = format!("{:X}", hash.result());
        let route = format!("/range/{:.5}", hash);

        future::Either::A(
            // Make API request.
            client
                .send(Get::new("https://api.pwnedpasswords.com", route))
                .map_err(DriverError::ActixMailbox)
                .and_then(|res| res.map_err(DriverError::Client))
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
        future::Either::B(future::err(DriverError::PwnedPasswordsDisabled))
    }
}
