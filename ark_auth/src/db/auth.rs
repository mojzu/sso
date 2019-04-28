use crate::db::user;
use crate::db::{DbError, TokenData};
use crate::models::{AuthKey, AuthService, AuthUser};
use diesel::prelude::*;
use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    sub: String,
    exp: usize,
    password_revision: Option<i64>,
}

impl Claims {
    pub fn new(iss: i64, sub: i64, exp: i64) -> Self {
        let dt = chrono::Utc::now();
        let exp = dt.timestamp() as usize + exp as usize;
        Claims {
            iss: iss.to_string(),
            sub: sub.to_string(),
            exp,
            password_revision: None,
        }
    }

    pub fn set_password_revision(mut self, password_revision: Option<i64>) -> Self {
        self.password_revision = password_revision;
        self
    }

    pub fn password_revision(&self) -> Option<i64> {
        self.password_revision
    }

    pub fn validation(iss: i64, sub: i64) -> Validation {
        Validation {
            iss: Some(iss.to_string()),
            sub: Some(sub.to_string()),
            ..Validation::default()
        }
    }
}

pub fn login(user: &AuthUser, key: &AuthKey, service: &AuthService) -> Result<TokenData, DbError> {
    let claims = Claims::new(service.service_id, user.user_id, 3600);
    let token = encode(&Header::default(), &claims, key.key_value.as_ref())
        .map_err(DbError::Jsonwebtoken)?;

    Ok(TokenData {
        user_id: user.user_id,
        token,
        token_expires: claims.exp,
    })
}

pub fn reset_password(
    user: &AuthUser,
    key: &AuthKey,
    service: &AuthService,
) -> Result<TokenData, DbError> {
    let password_revision = match user.user_password_revision {
        Some(password_revision) => Ok(password_revision),
        None => Err(DbError::InvalidPasswordRevision),
    }?;
    let claims = Claims::new(service.service_id, user.user_id, 3600)
        .set_password_revision(Some(password_revision));
    let token = encode(&Header::default(), &claims, key.key_value.as_ref())
        .map_err(DbError::Jsonwebtoken)?;

    Ok(TokenData {
        user_id: user.user_id,
        token,
        token_expires: claims.exp,
    })
}

pub fn reset_password_confirm(
    token: &str,
    password: &str,
    user: &AuthUser,
    key: &AuthKey,
    service: &AuthService,
    conn: &PgConnection,
) -> Result<usize, DbError> {
    let validation = Claims::validation(service.service_id, user.user_id);
    let data = decode::<Claims>(token, key.key_value.as_ref(), &validation)
        .map_err(DbError::Jsonwebtoken)?;

    let password_revision = data
        .claims
        .password_revision()
        .ok_or(DbError::InvalidPasswordRevision)?;
    user::update_password_by_id(user.user_id, password, password_revision, conn)
}

pub fn token_verify(
    token: &str,
    user: &AuthUser,
    key: &AuthKey,
    service: &AuthService,
) -> Result<TokenData, DbError> {
    let validation = Claims::validation(service.service_id, user.user_id);
    let data = decode::<Claims>(token, key.key_value.as_ref(), &validation)
        .map_err(DbError::Jsonwebtoken)?;

    Ok(TokenData {
        user_id: user.user_id,
        token: token.to_owned(),
        token_expires: data.claims.exp,
    })
}

pub fn token_refresh(
    token: &str,
    user: &AuthUser,
    key: &AuthKey,
    service: &AuthService,
) -> Result<TokenData, DbError> {
    let validation = Claims::validation(service.service_id, user.user_id);
    let _ = decode::<Claims>(token, key.key_value.as_ref(), &validation)
        .map_err(|_e| DbError::Unwrap("failed to decode jwt"))?;
    login(user, key, service).map(Into::into)
}

/// Unsafely decodes a token, checks if service identifier matches `iss` claim.
/// If matched, returns the `sub` claim, which may be a user identifier.
pub fn token_unsafe_decode(token: &str, service_id: i64) -> Result<i64, DbError> {
    let claims: Claims = dangerous_unsafe_decode(token)
        .map_err(|_e| DbError::Unwrap("failed to decode jwt"))?
        .claims;
    let iss = claims
        .iss
        .parse::<i64>()
        .map_err(|_e| DbError::Unwrap("failed to parse i64"))?;
    let sub = claims
        .sub
        .parse::<i64>()
        .map_err(|_e| DbError::Unwrap("failed to parse i64"))?;

    if service_id != iss {
        return Err(DbError::NotFound);
    }
    Ok(sub)
}
