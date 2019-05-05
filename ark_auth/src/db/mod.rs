pub mod auth;
pub mod user;

use crate::api;
use crate::models::{AuthCsrf, AuthKey, AuthService, AuthUser};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

#[derive(Fail, Debug)]
pub enum DbError {
    #[fail(display = "DbError::Unwrap {}", _0)]
    Unwrap(&'static str),
    #[fail(display = "DbError::InvalidOrder")]
    InvalidOrder,
    #[fail(display = "DbError::InvalidPassword")]
    InvalidPassword,
    #[fail(display = "DbError::InvalidPasswordRevision")]
    InvalidPasswordRevision,
    #[fail(display = "DbError::NotFound")]
    NotFound,

    #[fail(display = "DbError::Diesel {}", _0)]
    Diesel(#[fail(cause)] diesel::result::Error),
    #[fail(display = "DbError::R2d2 {}", _0)]
    R2d2(#[fail(cause)] r2d2::Error),
    #[fail(display = "DbError::Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),
}

impl Db {

    pub fn oauth2_login(&self, email: &str, service_id: i64) -> Result<TokenData, DbError> {
        let conn = self.connection()?;
        let user = user::read_by_email(email, &conn)?;
        let service = service::read_by_id(service_id, service_id, &conn)?;
        let key = key::read_by_user_id(user.user_id, service_id, &conn)?;
        auth::login(&user, &key, &service).map(Into::into)
    }

    pub fn auth_login(
        &self,
        email: &str,
        password: &str,
        service: &AuthService,
    ) -> Result<TokenData, DbError> {
        let conn = self.connection()?;
        let user = user::read_by_email(email, &conn)?;
        user::check_password(user.user_password.as_ref().map(|x| &**x), password)?;
        let key = key::read_by_user_id(user.user_id, service.service_id, &conn)?;
        auth::login(&user, &key, service)
    }

    pub fn auth_reset_password(
        &self,
        email: &str,
        service: &AuthService,
    ) -> Result<(AuthUser, TokenData), DbError> {
        let conn = self.connection()?;
        let user = user::read_by_email(email, &conn)?;
        let key = key::read_by_user_id(user.user_id, service.service_id, &conn)?;
        let token = auth::reset_password(&user, &key, service)?;
        Ok((user, token))
    }

    pub fn auth_reset_password_confirm(
        &self,
        token: &str,
        password: &str,
        service: &AuthService,
    ) -> Result<usize, DbError> {
        // Unsafely decode token to get user identifier, read key for doing safe token decode.
        let user_id = auth::token_unsafe_decode(token, service.service_id)?;
        let conn = self.connection()?;
        let key = key::read_by_user_id(user_id, service.service_id, &conn)?;
        let user = user::read_by_id(user_id, &conn)?;
        auth::reset_password_confirm(token, password, &user, &key, service, &conn)
    }

    pub fn auth_token_verify(
        &self,
        token: &str,
        service: &AuthService,
    ) -> Result<TokenData, DbError> {
        // Unsafely decode token to get user identifier, read key for doing safe token decode.
        let user_id = auth::token_unsafe_decode(token, service.service_id)?;
        let conn = self.connection()?;
        let key = key::read_by_user_id(user_id, service.service_id, &conn)?;
        let user = user::read_by_id(user_id, &conn)?;
        auth::token_verify(token, &user, &key, service)
    }

    pub fn auth_token_refresh(
        &self,
        token: &str,
        service: &AuthService,
    ) -> Result<TokenData, DbError> {
        // Unsafely decode token to get user identifier, read key for doing safe token decode.
        let user_id = auth::token_unsafe_decode(token, service.service_id)?;
        let conn = self.connection()?;
        let key = key::read_by_user_id(user_id, service.service_id, &conn)?;
        let user = user::read_by_id(user_id, &conn)?;
        auth::token_refresh(token, &user, &key, service)
    }

    pub fn auth_token_revoke(&self, token: &str, service: &AuthService) -> Result<(), DbError> {
        // Unsafely decode token to get user identifier, read key for doing safe token decode.
        let user_id = auth::token_unsafe_decode(token, service.service_id)?;
        let conn = self.connection()?;
        let key = key::read_by_user_id(user_id, service.service_id, &conn)?;
        key::delete_by_id(key.key_id, service.service_id, &conn)?;
        Ok(())
    }

    pub fn auth_key_verify(
        &self,
        key_value: &str,
        service: &AuthService,
    ) -> Result<KeyData, DbError> {
        let conn = self.connection()?;
        let key = key::user_read_by_value(key_value, service.service_id, &conn)?;
        let user_id = key.user_id.unwrap();
        Ok(KeyData {
            user_id,
            key: key_value.to_owned(),
        })
    }

    pub fn auth_key_revoke(&self, key_value: &str, service: &AuthService) -> Result<(), DbError> {
        let conn = self.connection()?;
        let key = key::user_read_by_value(key_value, service.service_id, &conn)?;
        key::delete_by_id(key.key_id, service.service_id, &conn)?;
        Ok(())
    }

    pub fn user_read_by_email(&self, email: &str) -> Result<AuthUser, DbError> {
        let conn = self.connection()?;
        user::read_by_email(email, &conn)
    }
}
