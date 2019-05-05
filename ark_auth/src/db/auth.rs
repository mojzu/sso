use crate::db::user;
use crate::db::{DbError, TokenData};
use crate::models::{AuthKey, AuthService, AuthUser};
use diesel::prelude::*;
use jsonwebtoken::{dangerous_unsafe_decode, decode, encode, Header, Validation};

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
