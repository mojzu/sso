use crate::{
    core,
    core::{Error, Service, User, UserKey, UserToken},
    driver,
};

// TODO(feature): Warning logs for bad requests.

/// User authentication using email address and password.
pub fn login(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
    password: &str,
    token_exp: i64,
) -> Result<UserToken, Error> {
    let user = user_read_by_email(driver, Some(service), email)?;
    core::check_password(user.password_hash.as_ref().map(|x| &**x), &password)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::encode_user_token(service.id, user.id, &key.value, token_exp)
}

/// User reset password request.
pub fn reset_password(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
    token_exp: i64,
) -> Result<(User, UserToken), Error> {
    let user = user_read_by_email(driver, Some(service), email)?;
    let password_revision = user.password_revision.ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;

    let reset_token = jwt::encode_reset_token(
        service.id,
        user.id,
        password_revision,
        &key.value,
        token_exp,
    )?;
    Ok((user, reset_token))
}

/// User reset password confirm.
pub fn reset_password_confirm(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, service.id)?;
    let user =
        core::user::read_by_id(driver, Some(service), user_id)?.ok_or_else(|| Error::BadRequest)?;
    let user_password_revision = user.password_revision.ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    let password_revision = jwt::decode_reset_token(service.id, user.id, &key.value, token)?;

    // If password revisions do not match, token has been used or password has been changed.
    if password_revision != user_password_revision {
        Err(Error::BadRequest)
    } else {
        core::user::update_password_by_id(
            driver,
            Some(service),
            user.id,
            password,
            password_revision,
        )
    }
}

/// Verify user key.
pub fn key_verify(driver: &driver::Driver, service: &Service, key: &str) -> Result<UserKey, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;
    let user_id = key.user_id.ok_or_else(|| Error::BadRequest)?;
    Ok(UserKey {
        user_id,
        key: key.value,
    })
}

/// Revoke user key.
pub fn key_revoke(driver: &driver::Driver, service: &Service, key: &str) -> Result<usize, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;
    core::key::delete_by_id(driver, Some(service), key.id)
}

/// Verify user token.
pub fn token_verify(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, service.id)?;
    let user =
        core::user::read_by_id(driver, Some(service), user_id)?.ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::decode_user_token(service.id, user.id, &key.value, token)
}

/// Refresh user token.
pub fn token_refresh(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
    token_exp: i64,
) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, service.id)?;
    let user =
        core::user::read_by_id(driver, Some(service), user_id)?.ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    jwt::decode_user_token(service.id, user.id, &key.value, token)?;
    jwt::encode_user_token(service.id, user.id, &key.value, token_exp)
}

/// Revoke user token.
pub fn token_revoke(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let user_id = jwt::decode_unsafe(token, service.id)?;
    let user =
        core::user::read_by_id(driver, Some(service), user_id)?.ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;
    core::key::delete_by_id(driver, Some(service), key.id)
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &driver::Driver,
    service_id: i64,
    email: &str,
    token_exp: i64,
) -> Result<(Service, UserToken), Error> {
    let service = driver
        .service_read_by_id(service_id)
        .map_err(Error::Driver)?
        .ok_or_else(|| Error::BadRequest)?;
    let user = user_read_by_email(driver, Some(&service), email)?;
    let key = core::key::read_by_user(driver, &service, &user)?.ok_or_else(|| Error::BadRequest)?;

    let user_token = jwt::encode_user_token(service.id, user.id, &key.value, token_exp)?;
    Ok((service, user_token))
}

/// Read user by email address.
/// Also checks user is active, returns bad request if inactive.
fn user_read_by_email(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    email: &str,
) -> Result<User, Error> {
    let user =
        core::user::read_by_email(driver, service_mask, email)?.ok_or_else(|| Error::BadRequest)?;
    if !user.active {
        return Err(Error::BadRequest);
    }
    Ok(user)
}

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
        pub fn new(iss: i64, sub: i64, exp: i64) -> Self {
            let dt = chrono::Utc::now();
            let exp = dt.timestamp() as usize + exp as usize;
            Claims {
                iss: iss.to_string(),
                sub: sub.to_string(),
                exp,
            }
        }

        pub fn validation(iss: i64, sub: i64) -> Validation {
            Validation {
                iss: Some(iss.to_string()),
                sub: Some(sub.to_string()),
                ..Validation::default()
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResetClaims {
        iss: String,
        sub: String,
        exp: usize,
        password_revision: i64,
    }

    impl ResetClaims {
        pub fn new(iss: i64, sub: i64, exp: i64, password_revision: i64) -> Self {
            let dt = chrono::Utc::now();
            let exp = dt.timestamp() as usize + exp as usize;
            ResetClaims {
                iss: iss.to_string(),
                sub: sub.to_string(),
                exp,
                password_revision,
            }
        }

        pub fn validation(iss: i64, sub: i64) -> Validation {
            Validation {
                iss: Some(iss.to_string()),
                sub: Some(sub.to_string()),
                ..Validation::default()
            }
        }
    }

    pub fn encode_user_token(
        service_id: i64,
        user_id: i64,
        key_value: &str,
        exp: i64,
    ) -> Result<UserToken, Error> {
        let claims = Claims::new(service_id, user_id, exp);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id,
            token,
            token_expires: claims.exp,
        })
    }

    /// Safely decodes a user token.
    pub fn decode_user_token(
        service_id: i64,
        user_id: i64,
        key_value: &str,
        token: &str,
    ) -> Result<UserToken, Error> {
        let validation = Claims::validation(service_id, user_id);
        let data = decode::<Claims>(token, key_value.as_bytes(), &validation)
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id,
            token: token.to_owned(),
            token_expires: data.claims.exp,
        })
    }

    pub fn encode_reset_token(
        service_id: i64,
        user_id: i64,
        password_revision: i64,
        key_value: &str,
        exp: i64,
    ) -> Result<UserToken, Error> {
        let claims = ResetClaims::new(service_id, user_id, exp, password_revision);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id,
            token,
            token_expires: claims.exp,
        })
    }

    /// Safely decodes a reset token and returns the password revision.
    pub fn decode_reset_token(
        service_id: i64,
        user_id: i64,
        key_value: &str,
        token: &str,
    ) -> Result<i64, Error> {
        let validation = ResetClaims::validation(service_id, user_id);
        let data = decode::<ResetClaims>(token, key_value.as_bytes(), &validation)
            .map_err(Error::Jsonwebtoken)?;
        Ok(data.claims.password_revision)
    }

    /// Unsafely decodes a token, checks if service ID matches `iss` claim.
    /// If matched, returns the `sub` claim, which may be a user ID.
    /// The user ID must then be used safely decode the token to proceed.
    pub fn decode_unsafe(token: &str, service_id: i64) -> Result<i64, Error> {
        let claims: Claims = dangerous_unsafe_decode(token)
            .map_err(Error::Jsonwebtoken)?
            .claims;
        let issuer = claims
            .iss
            .parse::<i64>()
            .map_err(|_err| Error::BadRequest)?;
        let subject = claims
            .sub
            .parse::<i64>()
            .map_err(|_err| Error::BadRequest)?;

        if service_id != issuer {
            return Err(Error::BadRequest);
        }
        Ok(subject)
    }
}
