use crate::core;
use crate::core::{Error, Service, User, UserKey, UserToken};
use crate::driver::Driver;

// TODO(feature): Warning logs for bad requests.

/// User authentication using email address and password.
pub fn login(
    driver: &Driver,
    service: &Service,
    email: &str,
    password: &str,
    token_expires: usize,
) -> Result<UserToken, Error> {
    let user = user_read_by_email(driver, Some(service), email)?;
    core::check_password(user.password_hash.as_ref().map(|x| &**x), &password)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::encode_user_token(&service.id, &user.id, &key.value, token_expires)
}

/// User reset password request.
pub fn reset_password(
    driver: &Driver,
    service: &Service,
    email: &str,
    token_expires: usize,
) -> Result<(User, UserToken), Error> {
    let user = user_read_by_email(driver, Some(service), email)?;
    let csrf_key = uuid::Uuid::new_v4().to_simple().to_string();
    let csrf = core::csrf::create(driver, service, &csrf_key, &csrf_key)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;

    let reset_token =
        jwt::encode_reset_token(&service.id, &user.id, &csrf.key, &key.value, token_expires)?;
    Ok((user, reset_token))
}

/// User reset password confirm.
pub fn reset_password_confirm(
    driver: &Driver,
    service: &Service,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, &service.id)?;
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    let csrf_key = jwt::decode_reset_token(&service.id, &user.id, &key.value, token)?;
    let _csrf = core::csrf::read_by_key(driver, &csrf_key)?.ok_or_else(|| Error::BadRequest)?;

    core::user::update_password_by_id(driver, Some(service), &user.id, password)
}

/// User update email request.
pub fn update_email(
    driver: &Driver,
    service: &Service,
    token: Option<&str>,
    key: Option<&str>,
    email: &str,
    token_expires: usize,
) -> Result<(User, String, String), Error> {
    let user_id = key_or_token_verify(driver, service, key, token)?;
    unimplemented!();
}

/// User update email revoke request.
pub fn update_email_revoke(driver: &Driver, service: &Service, token: &str) -> Result<(), Error> {
    unimplemented!();
}

/// User update password request.
pub fn update_password(
    driver: &Driver,
    service: &Service,
    token: Option<&str>,
    key: Option<&str>,
    password: &str,
    token_expires: usize,
) -> Result<(User, String), Error> {
    let user_id = key_or_token_verify(driver, service, key, token)?;
    unimplemented!();
}

/// User update password revoke request.
pub fn update_password_revoke(
    driver: &Driver,
    service: &Service,
    token: &str,
) -> Result<(), Error> {
    unimplemented!();
}

/// Verify user key.
pub fn key_verify(driver: &Driver, service: &Service, key: &str) -> Result<UserKey, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;
    let user_id = key.user_id.ok_or_else(|| Error::BadRequest)?;
    Ok(UserKey {
        user_id,
        key: key.value,
    })
}

/// Revoke user key.
pub fn key_revoke(driver: &Driver, service: &Service, key: &str) -> Result<usize, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;
    core::key::delete_by_id(driver, Some(service), &key.id)
}

/// Verify user token.
pub fn token_verify(driver: &Driver, service: &Service, token: &str) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, &service.id)?;
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::decode_user_token(&service.id, &user.id, &key.value, token)
}

/// Refresh user token.
pub fn token_refresh(
    driver: &Driver,
    service: &Service,
    token: &str,
    token_expires: usize,
) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, &service.id)?;
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::decode_user_token(&service.id, &user.id, &key.value, token)?;
    jwt::encode_user_token(&service.id, &user.id, &key.value, token_expires)
}

/// Revoke user token.
pub fn token_revoke(driver: &Driver, service: &Service, token: &str) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, &service.id)?;
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::decode_user_token(&service.id, &user.id, &key.value, token)?;
    core::key::delete_by_id(driver, Some(service), &key.id)
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &Driver,
    service_id: &str,
    email: &str,
    token_expires: usize,
) -> Result<(Service, UserToken), Error> {
    let service = driver
        .service_read_by_id(service_id)
        .map_err(Error::Driver)?
        .ok_or_else(|| Error::BadRequest)?;
    let user = user_read_by_email(driver, Some(&service), email)?;
    let key = core::key::read_by_user(driver, &service, &user)?.ok_or_else(|| Error::BadRequest)?;

    let user_token = jwt::encode_user_token(&service.id, &user.id, &key.value, token_expires)?;
    Ok((service, user_token))
}

/// Read user by email address.
/// Also checks user is active, returns bad request if inactive.
fn user_read_by_email(
    driver: &Driver,
    service_mask: Option<&Service>,
    email: &str,
) -> Result<User, Error> {
    let user =
        core::user::read_by_email(driver, service_mask, email)?.ok_or_else(|| Error::BadRequest)?;
    if !user.is_active {
        return Err(Error::BadRequest);
    }
    Ok(user)
}

/// Get user ID from valid key or token.
fn key_or_token_verify(
    driver: &Driver,
    service: &Service,
    key: Option<&str>,
    token: Option<&str>,
) -> Result<String, Error> {
    match key {
        Some(key) => {
            let user_key = key_verify(driver, service, key)?;
            Ok(user_key.user_id)
        }
        None => match token {
            Some(token) => {
                let user_token = token_verify(driver, service, token)?;
                Ok(user_token.user_id)
            }
            None => Err(Error::Forbidden),
        },
    }
}

// TODO(refactor): Check is_active flag here.
// TODO(refactor): Improve handling of decode errors.
// ServerError::Core CoreError::Jsonwebtoken invalid signature

mod jwt {
    use crate::core::{Error, UserToken};
    use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        iss: String,
        sub: String,
        exp: usize,
    }

    impl Claims {
        pub fn new(iss: &str, sub: &str, exp: usize) -> Self {
            let dt = chrono::Utc::now();
            let exp = dt.timestamp() as usize + exp as usize;
            Claims {
                iss: iss.to_owned(),
                sub: sub.to_owned(),
                exp,
            }
        }

        pub fn validation(iss: &str, sub: &str) -> Validation {
            Validation {
                iss: Some(iss.to_owned()),
                sub: Some(sub.to_owned()),
                ..Validation::default()
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResetClaims {
        iss: String,
        sub: String,
        exp: usize,
        csrf: String,
    }

    impl ResetClaims {
        pub fn new(iss: &str, sub: &str, exp: usize, csrf: &str) -> Self {
            let dt = chrono::Utc::now();
            let exp = dt.timestamp() as usize + exp as usize;
            ResetClaims {
                iss: iss.to_owned(),
                sub: sub.to_owned(),
                exp,
                csrf: csrf.to_owned(),
            }
        }

        pub fn validation(iss: &str, sub: &str) -> Validation {
            Validation {
                iss: Some(iss.to_owned()),
                sub: Some(sub.to_owned()),
                ..Validation::default()
            }
        }
    }

    pub fn encode_user_token(
        service_id: &str,
        user_id: &str,
        key_value: &str,
        exp: usize,
    ) -> Result<UserToken, Error> {
        let claims = Claims::new(service_id, user_id, exp);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id: user_id.to_owned(),
            token,
            token_expires: claims.exp,
        })
    }

    /// Safely decodes a user token.
    pub fn decode_user_token(
        service_id: &str,
        user_id: &str,
        key_value: &str,
        token: &str,
    ) -> Result<UserToken, Error> {
        let validation = Claims::validation(service_id, user_id);
        let data = decode::<Claims>(token, key_value.as_bytes(), &validation)
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id: user_id.to_owned(),
            token: token.to_owned(),
            token_expires: data.claims.exp,
        })
    }

    pub fn encode_reset_token(
        service_id: &str,
        user_id: &str,
        csrf: &str,
        key_value: &str,
        exp: usize,
    ) -> Result<UserToken, Error> {
        let claims = ResetClaims::new(service_id, user_id, exp, csrf);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id: user_id.to_owned(),
            token,
            token_expires: claims.exp,
        })
    }

    /// Safely decodes a reset token and returns the csrf value.
    pub fn decode_reset_token(
        service_id: &str,
        user_id: &str,
        key_value: &str,
        token: &str,
    ) -> Result<String, Error> {
        let validation = ResetClaims::validation(service_id, user_id);
        let data = decode::<ResetClaims>(token, key_value.as_bytes(), &validation)
            .map_err(Error::Jsonwebtoken)?;
        Ok(data.claims.csrf)
    }

    /// Unsafely decodes a token, checks if service ID matches `iss` claim.
    /// If matched, returns the `sub` claim, which may be a user ID.
    /// The user ID must then be used safely decode the token to proceed.
    pub fn decode_unsafe(token: &str, service_id: &str) -> Result<String, Error> {
        let claims: Claims = dangerous_unsafe_decode(token)
            .map_err(Error::Jsonwebtoken)?
            .claims;
        if service_id != claims.iss {
            return Err(Error::BadRequest);
        }
        Ok(claims.sub)
    }
}
