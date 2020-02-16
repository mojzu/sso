//! # Pattern functions.
use crate::{
    AuditBuilder, DriverError, DriverResult, HeaderAuth, HeaderAuthType, Jwt, KeyRead, KeyType,
    KeyWithValue, Postgres, Service, ServiceRead, User, UserPasswordMeta, UserRead,
};
use libreauth::oath::TOTPBuilder;
use reqwest::Client;
use sha1::{Digest, Sha1};
use url::Url;
use uuid::Uuid;

// TODO(sam,refactor): Improve usability, composability of pattern functions, diesel async?

/// Verify TOTP code using key.
pub fn totp_verify(key: &str, code: &str) -> DriverResult<()> {
    let totp = TOTPBuilder::new()
        .base32_key(key)
        .finalize()
        .map_err::<DriverError, _>(Into::into)?;

    if !totp.is_valid(&code) {
        Err(DriverError::TotpInvalid)
    } else {
        Ok(())
    }
}

/// Authenticate root key.
pub fn key_root_authenticate(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    auth: &HeaderAuth,
) -> DriverResult<()> {
    let key = match auth {
        HeaderAuth::Traefik(x) => match x.key_id {
            Some(key_id) => {
                let key = driver.key_read(&KeyRead::RootId(key_id), None)?;
                Ok(key)
            }
            None => Err(DriverError::KeyUndefined),
        },
        HeaderAuth::Header(x) => match x {
            HeaderAuthType::Key(x) => {
                let key = driver.key_read(&KeyRead::RootValue(x.to_owned()), None)?;
                Ok(key)
            }
            _ => Err(DriverError::KeyUndefined),
        },
        HeaderAuth::None => Err(DriverError::KeyUndefined),
    }?;
    key.ok_or_else(|| DriverError::KeyNotFound).map(|key| {
        audit.key(Some(&key));
        ()
    })
}

/// Authenticate service key.
///
/// If audit meta user is some, this function will also verify
/// the user key or token to authenticate this request.
pub fn key_service_authenticate(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    auth: &HeaderAuth,
) -> DriverResult<Service> {
    let service = key_service_authenticate_try(driver, audit, auth)?;
    check_audit_user(driver, audit, &service)?;
    Ok(service)
}

pub fn user_key_token_authenticate(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    user_auth: &HeaderAuth,
    service_key: Option<String>,
) -> DriverResult<User> {
    match service_key {
        Some(service_key) => {
            let service_auth = HeaderAuth::Header(HeaderAuthType::Key(service_key));
            let service = key_service_authenticate(driver, audit, &service_auth)?;

            match user_auth {
                HeaderAuth::Header(x) => match x {
                    HeaderAuthType::Key(x) => {
                        // Key verify requires key key type.
                        let key =
                            key_read_user_value_checked(driver, &service, audit, x, KeyType::Key)?;
                        let user = user_read_id_checked(
                            driver,
                            Some(&service),
                            audit,
                            key.user_id.unwrap(),
                        )?;
                        Ok(user)
                    }
                    HeaderAuthType::Token(x) => {
                        // Unsafely decode token to get user identifier, used to read key for safe token decode.
                        let (user_id, _) = Jwt::decode_unsafe(x, service.id)?;

                        // Token verify requires token key type.
                        let user = user_read_id_checked(driver, Some(&service), audit, user_id)?;
                        let key =
                            key_read_user_checked(driver, &service, audit, &user, KeyType::Token)?;

                        // Safely decode token with user key.
                        Jwt::decode_access_token(&service, &user, &key, x)?;
                        Ok(user)
                    }
                },
                _ => Err(DriverError::KeyUndefined),
            }
        }
        None => Err(DriverError::KeyUndefined),
    }
}

/// Authenticate service or root key.
///
/// If audit meta user is some, this function will also verify
/// the user key or token to authenticate this request.
pub fn key_authenticate(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    auth: &HeaderAuth,
) -> DriverResult<Option<Service>> {
    let service = key_service_authenticate_try(driver, audit, auth)
        .and_then(|service| {
            check_audit_user(driver, audit, &service)?;
            Ok(service)
        })
        .map(Some)
        .or_else(|_err| key_root_authenticate(driver, audit, auth).map(|_| None))?;
    Ok(service)
}

fn key_service_authenticate_try(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    auth: &HeaderAuth,
) -> DriverResult<Service> {
    let key = match auth {
        HeaderAuth::Traefik(x) => match (x.key_id, x.service_id) {
            (Some(key_id), Some(service_id)) => {
                let key = driver.key_read(&KeyRead::ServiceId(service_id, key_id), None)?;
                Ok(key)
            }
            _ => Err(DriverError::KeyUndefined),
        },
        HeaderAuth::Header(x) => match x {
            HeaderAuthType::Key(x) => {
                let key = driver.key_read(&KeyRead::ServiceValue(x.to_owned()), None)?;
                Ok(key)
            }
            _ => Err(DriverError::KeyUndefined),
        },
        HeaderAuth::None => Err(DriverError::KeyUndefined),
    }?;
    key.ok_or_else(|| DriverError::KeyNotFound)
        .and_then(|key| {
            audit.key(Some(&key));
            key.service_id
                .ok_or_else(|| DriverError::KeyServiceUndefined)
        })
        .and_then(|service_id| key_service_authenticate_inner(driver, audit, service_id))
}

fn key_service_authenticate_inner(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    service_id: Uuid,
) -> DriverResult<Service> {
    let service = driver
        .service_read(&ServiceRead::new(service_id), None)?
        .ok_or_else(|| DriverError::ServiceNotFound)?
        .check()?;
    audit.service(Some(&service));
    Ok(service)
}

fn check_audit_user(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    service: &Service,
) -> DriverResult<()> {
    let user = audit.meta().user().cloned();
    match user {
        // TODO(sam,refactor): Duplicate authentication code with api module, refactor.
        Some(user) => match user {
            HeaderAuthType::Key(key_value) => {
                // Key verify requires key key type.
                let key =
                    key_read_user_value_checked(driver, &service, audit, key_value, KeyType::Key)?;
                user_read_id_checked(driver, Some(&service), audit, key.user_id.unwrap())?;
                Ok(())
            }
            HeaderAuthType::Token(token) => {
                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) = Jwt::decode_unsafe(&token, service.id)?;

                // Token verify requires token key type.
                let user = user_read_id_checked(driver, Some(&service), audit, user_id)?;
                let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)?;

                // Safely decode token with user key.
                Jwt::decode_access_token(&service, &user, &key, &token)?;
                Ok(())
            }
        },
        None => Ok(()),
    }
}

/// Read user by ID.
/// Checks user is enabled, returns bad request if disabled.
pub fn user_read_id_checked(
    driver: &Postgres,
    _service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    id: Uuid,
) -> DriverResult<User> {
    let read = UserRead::Id(id);
    let user = driver
        .user_read(&read)?
        .ok_or_else(|| DriverError::UserNotFound)?;
    audit.user(Some(&user));
    if !user.is_enabled {
        return Err(DriverError::UserDisabled);
    }
    Ok(user)
}

/// Unchecked read user by ID.
/// Does not check user is enabled.
pub fn user_read_id_unchecked(
    driver: &Postgres,
    _service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    id: Uuid,
) -> DriverResult<User> {
    let read = UserRead::Id(id);
    let user = driver
        .user_read(&read)?
        .ok_or_else(|| DriverError::UserNotFound)?;
    audit.user(Some(&user));
    Ok(user)
}

/// Read user by email address.
/// Also checks user is enabled, returns bad request if disabled.
pub fn user_read_email_checked(
    driver: &Postgres,
    _service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    email: &str,
) -> DriverResult<User> {
    let read = UserRead::Email(email.to_owned());
    let user = driver
        .user_read(&read)?
        .ok_or_else(|| DriverError::UserNotFound)?;
    audit.user(Some(&user));
    if !user.is_enabled {
        return Err(DriverError::UserDisabled);
    }
    Ok(user)
}

/// Read key by user reference and key type.
/// Also checks key is enabled and not revoked, returns bad request if disabled.
pub fn key_read_user_checked(
    driver: &Postgres,
    service: &Service,
    audit: &mut AuditBuilder,
    user: &User,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(
            &KeyRead::user_id(service.id, user.id, true, false, key_type),
            None,
        )?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    if !key.is_enabled {
        Err(DriverError::KeyDisabled)
    } else if key.is_revoked {
        Err(DriverError::KeyRevoked)
    } else {
        Ok(key)
    }
}

/// Unchecked read key by user reference.
/// Does not check key is enabled or not revoked.
pub fn key_read_user_unchecked(
    driver: &Postgres,
    service: &Service,
    audit: &mut AuditBuilder,
    user: &User,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(
            &KeyRead::user_id(service.id, user.id, true, false, key_type),
            None,
        )?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    Ok(key)
}

/// Read key by user value.
/// Also checks key is enabled and not revoked, returns bad request if disabled.
pub fn key_read_user_value_checked<K>(
    driver: &Postgres,
    service: &Service,
    audit: &mut AuditBuilder,
    key: K,
    key_type: KeyType,
) -> DriverResult<KeyWithValue>
where
    K: Into<String>,
{
    let key = driver
        .key_read(
            &KeyRead::user_value(service.id, key.into(), true, false, key_type),
            None,
        )?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    if !key.is_enabled {
        Err(DriverError::KeyDisabled)
    } else if key.is_revoked {
        Err(DriverError::KeyRevoked)
    } else {
        Ok(key)
    }
}

/// Unchecked read key by user value.
/// Does not check key is enabled and not revoked.
pub fn key_read_user_value_unchecked(
    driver: &Postgres,
    service: &Service,
    audit: &mut AuditBuilder,
    key: &str,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(
            &KeyRead::user_value(service.id, key, true, false, key_type),
            None,
        )?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    Ok(key)
}

/// Password strength and pwned checks.
///
/// If password is empty, returns 0 for strength and true for pwned.
/// If password is none, returns none for strength and pwned.
pub async fn password_meta(
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
            let password_pwned = match password_meta_pwned(client, enabled, password).await {
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
async fn password_meta_pwned(client: &Client, enabled: bool, password: &str) -> DriverResult<bool> {
    if enabled {
        // Make request to API using first 5 characters of SHA1 password hash.
        let mut hash = Sha1::new();
        hash.input(password);
        let hash = format!("{:X}", hash.result());
        let url = format!("https://api.pwnedpasswords.com/range/{:.5}", hash);

        match Url::parse(&url).map_err(DriverError::UrlParse) {
            Ok(url) => {
                let res = client.get(url).send().await.map_err(DriverError::Reqwest)?;
                let res = res.error_for_status().map_err(DriverError::Reqwest)?;
                let text = res.text().await.map_err(DriverError::Reqwest)?;

                // Compare suffix of hash to lines to determine if password is pwned.
                for line in text.lines() {
                    if hash[5..] == line[..35] {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Err(e) => Err(e),
        }
    } else {
        Err(DriverError::PwnedPasswordsDisabled)
    }
}
