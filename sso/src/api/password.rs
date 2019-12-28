use crate::{DriverError, DriverResult, UserPasswordMeta};
use reqwest::Client;
use sha1::{Digest, Sha1};
use url::Url;

/// Password strength and pwned checks.
///
/// If password is empty, returns 0 for strength and true for pwned.
/// If password is none, returns none for strength and pwned.
pub fn password_meta(
    client: &Client,
    enabled: bool,
    password: Option<String>,
) -> DriverResult<UserPasswordMeta> {
    match password.as_ref().map(|x| &**x) {
        Some("") => Ok(UserPasswordMeta::invalid()),
        Some(password) => {
            let password_strength = match password_meta_strength(password) {
                Ok(entropy) => Some(entropy.score()),
                Err(err) => {
                    warn!("{}", err);
                    None
                }
            };
            let password_pwned = match password_meta_pwned(client, enabled, password) {
                Ok(password_pwned) => Some(password_pwned),
                Err(err) => {
                    warn!("{}", err);
                    None
                }
            };
            Ok(UserPasswordMeta {
                password_strength,
                password_pwned,
            })
        }
        None => Ok(UserPasswordMeta::default()),
    }
}

/// Returns password strength test performed by `zxcvbn`.
/// <https://github.com/shssoichiro/zxcvbn-rs>
fn password_meta_strength(password: &str) -> DriverResult<zxcvbn::Entropy> {
    zxcvbn::zxcvbn(password, &[]).map_err(DriverError::Zxcvbn)
}

/// Returns true if password is present in `Pwned Passwords` index, else false.
/// <https://haveibeenpwned.com/Passwords>
fn password_meta_pwned(client: &Client, enabled: bool, password: &str) -> DriverResult<bool> {
    if enabled {
        // Make request to API using first 5 characters of SHA1 password hash.
        let mut hash = Sha1::new();
        hash.input(password);
        let hash = format!("{:X}", hash.result());
        let url = format!("https://api.pwnedpasswords.com/range/{:.5}", hash);

        match Url::parse(&url).map_err(DriverError::UrlParse) {
            Ok(url) => {
                client
                    .get(url)
                    .send()
                    .map_err(DriverError::Reqwest)
                    .and_then(|res| res.error_for_status().map_err(DriverError::Reqwest))
                    .and_then(|mut res| res.text().map_err(DriverError::Reqwest))
                    .and_then(move |text| {
                        // Compare suffix of hash to lines to determine if password is pwned.
                        for line in text.lines() {
                            if hash[5..] == line[..35] {
                                return Ok(true);
                            }
                        }
                        Ok(false)
                    })
            }
            Err(e) => Err(e),
        }
    } else {
        Err(DriverError::PwnedPasswordsDisabled)
    }
}
