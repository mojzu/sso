use crate::driver;
use crate::{
    core,
    core::{Error, Service, User, UserKey, UserToken},
};

/// User authentication using email address and password.
pub fn login(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
    password: &str,
) -> Result<UserToken, Error> {
    let user =
        core::user::read_by_email(driver, service, email)?.ok_or_else(|| Error::Forbidden)?;
    core::check_password(user.password_hash.as_ref().map(|x| &**x), &password)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::Forbidden)?;
    jwt::encode_user_token(service.id, user.id, &key.value)
}

/// User reset password request.
pub fn reset_password(
    driver: &driver::Driver,
    service: &Service,
    email: &str,
) -> Result<(User, UserToken), Error> {
    let user =
        core::user::read_by_email(driver, service, email)?.ok_or_else(|| Error::Forbidden)?;
    let password_revision = user.password_revision.ok_or_else(|| Error::Forbidden)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::Forbidden)?;

    let reset_token = jwt::encode_reset_token(service.id, user.id, password_revision, &key.value)?;
    Ok((user, reset_token))
}

/// User reset password confirm.
pub fn reset_password_confirm(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// Verify user key.
pub fn key_verify(driver: &driver::Driver, service: &Service, key: &str) -> Result<UserKey, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::Forbidden)?;
    let user_id = key.user_id.unwrap();
    Ok(UserKey {
        user_id,
        key: key.value,
    })
}

/// Revoke user key.
pub fn key_revoke(driver: &driver::Driver, service: &Service, key: &str) -> Result<usize, Error> {
    unimplemented!();
}

/// Verify user token.
pub fn token_verify(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// Refresh user token.
pub fn token_refresh(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<UserToken, Error> {
    unimplemented!();
}

/// Revoke user token.
pub fn token_revoke(
    driver: &driver::Driver,
    service: &Service,
    token: &str,
) -> Result<usize, Error> {
    unimplemented!();
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &driver::Driver,
    service_id: i64,
    email: &str,
) -> Result<(Service, UserToken), Error> {
    let service = driver
        .service_read_by_id(service_id)
        .map_err(Error::Driver)?
        .ok_or_else(|| Error::Forbidden)?;
    let user =
        core::user::read_by_email(driver, &service, email)?.ok_or_else(|| Error::Forbidden)?;
    let key = core::key::read_by_user(driver, &service, &user)?.ok_or_else(|| Error::Forbidden)?;

    let user_token = jwt::encode_user_token(service.id, user.id, &key.value)?;
    Ok((service, user_token))
}

mod jwt {
    use crate::core::{Error, UserToken};
    use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
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
    #[serde(deny_unknown_fields)]
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
    ) -> Result<UserToken, Error> {
        // TODO(feature): Configurable expiry, limit refreshes.
        let claims = Claims::new(service_id, user_id, 3600);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id,
            token,
            token_expires: claims.exp,
        })
    }

    pub fn encode_reset_token(
        service_id: i64,
        user_id: i64,
        password_revision: i64,
        key_value: &str,
    ) -> Result<UserToken, Error> {
        let claims = ResetClaims::new(service_id, user_id, 3600, password_revision);
        let token = encode(&Header::default(), &claims, key_value.as_bytes())
            .map_err(Error::Jsonwebtoken)?;

        Ok(UserToken {
            user_id,
            token,
            token_expires: claims.exp,
        })
    }
}

// TODO(refactor): Refactor this.
// pub fn login(
//     data: &web::Data<ApiData>,
//     email: &str,
//     service_id: i64,
// ) -> Result<(TokenData, AuthService), ApiError> {
//     let token = data
//         .db
//         .login(email, service_id)
//         .map_err(ApiError::Db)?;
//     let service = data
//         .db
//         .service_read_by_id(service_id, service_id)
//         .map_err(ApiError::Db)?;
//     Ok((token, service))
// }

// TODO(refactor): Refactor this.
// pub fn reset_password() {
// .and_then(|(service, (user, token_response))| {
//     // Send user email with reset password confirmation link.
//     match email::send_reset_password(
//         data.smtp(),
//         &user,
//         &service,
//         &token_response.token,
//     ) {
//         Ok(_) => Ok(token_response),
//         // Log warning in case of failure to send email.
//         Err(e) => {
//             warn!("Failed to send reset password email ({})", e);
//             Ok(token_response)
//         }
//     }
// })
// }

// Map invalid password, not found errors to bad request to prevent leakage.
// TODO(feature): Warning logs for bad requests.
// TODO(refactor): Refactor this.
// DbError::InvalidPassword | DbError::NotFound => ApiError::BadRequest,
