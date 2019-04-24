pub mod auth;
pub mod csrf;
pub mod key;
pub mod service;
pub mod user;

use crate::api::auth::{KeyResponse, TokenResponse};
use crate::models::{AuthCsrf, AuthKey, AuthService, AuthUser};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

// TODO(refactor): Diesel query clean up.
// TODO(feature): SQLite database support.
// TODO(feature): Password pwned, strength checks.

// Database migrations embedded in binary output for use in production.
embed_migrations!("migrations/postgres");

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
    #[fail(display = "DbError::Bcrypt {}", _0)]
    Bcrypt(#[fail(cause)] bcrypt::BcryptError),
}

impl From<diesel::result::Error> for DbError {
    fn from(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => DbError::NotFound,
            _ => DbError::Diesel(e),
        }
    }
}

/// Order by value enumeration.
pub enum DbOrder {
    Asc,
    Desc,
}

impl DbOrder {
    /// Parse value from string reference option.
    /// If value is none, returns ascending order by default.
    /// Else value is some, valid values are "asc" and "desc" only.
    pub fn parse(value: Option<&str>) -> Result<Self, DbError> {
        match value {
            Some(value) => match value {
                "asc" => Ok(DbOrder::Asc),
                "desc" => Ok(DbOrder::Desc),
                _ => Err(DbError::InvalidOrder),
            },
            None => Ok(DbOrder::Asc),
        }
    }
}

#[derive(Clone)]
pub struct Db {
    pool: r2d2::Pool<ConnectionManager<PgConnection>>,
}

type PooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

impl Db {
    pub fn new(url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = r2d2::Pool::builder().build(manager).unwrap();

        let conn = pool.get().unwrap();
        embedded_migrations::run(&conn).unwrap();

        Db { pool }
    }

    pub fn connection(&self) -> Result<PooledConnection, DbError> {
        self.pool.get().map_err(DbError::R2d2)
    }

    // Initialise database, creates one service with a key.
    pub fn init(&self, name: &str, url: &str) -> Result<(AuthService, AuthKey), DbError> {
        let conn = self.connection()?;
        let service = service::create(name, url, &conn)?;
        let key = key::create(name, service.service_id, None, &conn)?;
        Ok((service, key))
    }

    pub fn oauth_login(&self, email: &str, service_id: i64) -> Result<TokenResponse, DbError> {
        let conn = self.connection()?;
        let user = user::read_by_email(email, &conn)?;
        let service = service::read_by_id(service_id, service_id, &conn)?;
        let key = key::read_by_user_id(user.user_id, service_id, &conn)?;
        auth::login(&user, &key, &service)
    }

    pub fn auth_login(
        &self,
        email: &str,
        password: &str,
        service: &AuthService,
    ) -> Result<TokenResponse, DbError> {
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
    ) -> Result<(AuthUser, TokenResponse), DbError> {
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
    ) -> Result<TokenResponse, DbError> {
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
    ) -> Result<TokenResponse, DbError> {
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
    ) -> Result<KeyResponse, DbError> {
        let conn = self.connection()?;
        let key = key::user_read_by_value(key_value, service.service_id, &conn)?;
        let user_id = key.user_id.unwrap();
        Ok(KeyResponse {
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

    pub fn user_list(
        &self,
        offset: Option<i64>,
        limit: Option<i64>,
        order: Option<&str>,
    ) -> Result<Vec<AuthUser>, DbError> {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        let order = DbOrder::parse(order)?;

        let conn = self.connection()?;
        user::list(offset, limit, order, &conn)
    }

    pub fn user_create(
        &self,
        name: &str,
        email: &str,
        password: Option<&str>,
    ) -> Result<AuthUser, DbError> {
        let conn = self.connection()?;
        user::create(name, email, password, &conn)
    }

    pub fn user_read_by_id(&self, id: i64) -> Result<AuthUser, DbError> {
        let conn = self.connection()?;
        user::read_by_id(id, &conn)
    }

    pub fn user_read_by_email(&self, email: &str) -> Result<AuthUser, DbError> {
        let conn = self.connection()?;
        user::read_by_email(email, &conn)
    }

    pub fn user_update_by_id(&self, id: i64, name: Option<&str>) -> Result<AuthUser, DbError> {
        let conn = self.connection()?;
        user::update_by_id(id, name, &conn)
    }

    pub fn user_delete_by_id(&self, id: i64) -> Result<usize, DbError> {
        let conn = self.connection()?;
        user::delete_by_id(id, &conn)
    }

    pub fn service_list(
        &self,
        offset: Option<i64>,
        limit: Option<i64>,
        order: Option<&str>,
        service_id: i64,
    ) -> Result<Vec<AuthService>, DbError> {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        let order = DbOrder::parse(order)?;

        let conn = self.connection()?;
        service::list(offset, limit, order, service_id, &conn)
    }

    pub fn service_create(&self, name: &str, url: &str) -> Result<AuthService, DbError> {
        let conn = self.connection()?;
        service::create(name, url, &conn)
    }

    pub fn service_read_by_id(&self, id: i64, service_id: i64) -> Result<AuthService, DbError> {
        let conn = self.connection()?;
        service::read_by_id(id, service_id, &conn)
    }

    pub fn service_read_by_key_value(&self, key_value: &str) -> Result<AuthService, DbError> {
        let conn = self.connection()?;
        let key = key::service_read_by_value(key_value, &conn)?;
        service::read_by_id(key.service_id, key.service_id, &conn)
    }

    pub fn service_update_by_id(
        &self,
        id: i64,
        service_id: i64,
        name: Option<&str>,
    ) -> Result<AuthService, DbError> {
        let conn = self.connection()?;
        service::update_by_id(id, service_id, name, &conn)
    }

    pub fn service_delete_by_id(&self, id: i64, service_id: i64) -> Result<usize, DbError> {
        let conn = self.connection()?;
        service::delete_by_id(id, service_id, &conn)
    }

    pub fn key_list(
        &self,
        offset: Option<i64>,
        limit: Option<i64>,
        order: Option<&str>,
        service_id: i64,
    ) -> Result<Vec<AuthKey>, DbError> {
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        let order = DbOrder::parse(order)?;

        let conn = self.connection()?;
        key::list(offset, limit, order, service_id, &conn)
    }

    pub fn key_create(
        &self,
        name: &str,
        service_id: i64,
        user_id: Option<i64>,
    ) -> Result<AuthKey, DbError> {
        let conn = self.connection()?;
        key::create(name, service_id, user_id, &conn)
    }

    pub fn key_read_by_id(&self, id: i64, service_id: i64) -> Result<AuthKey, DbError> {
        let conn = self.connection()?;
        key::read_by_id(id, service_id, &conn)
    }

    pub fn key_update_by_id(
        &self,
        id: i64,
        service_id: i64,
        name: Option<&str>,
    ) -> Result<AuthKey, DbError> {
        let conn = self.connection()?;
        key::update_by_id(id, service_id, name, &conn)
    }

    pub fn key_delete_by_id(&self, id: i64, service_id: i64) -> Result<usize, DbError> {
        let conn = self.connection()?;
        key::delete_by_id(id, service_id, &conn)
    }

    pub fn csrf_create(
        &self,
        key: &str,
        value: &str,
        service_id: i64,
    ) -> Result<AuthCsrf, DbError> {
        let conn = self.connection()?;
        csrf::create(key, value, service_id, &conn)
    }

    pub fn csrf_read_by_key(&self, key: &str) -> Result<AuthCsrf, DbError> {
        let conn = self.connection()?;
        csrf::read_by_key(key, &conn)
    }
}
