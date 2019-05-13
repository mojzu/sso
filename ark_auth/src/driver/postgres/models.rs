use crate::{
    core,
    driver::postgres::schema::{auth_csrf, auth_key, auth_service, auth_user},
};
use chrono::{DateTime, Utc};

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_user"]
#[primary_key(user_id)]
pub struct AuthUser {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: i64,
    pub user_name: String,
    pub user_email: String,
    pub user_password_hash: Option<String>,
    pub user_password_revision: Option<i64>,
}

impl From<AuthUser> for core::User {
    fn from(user: AuthUser) -> Self {
        core::User {
            created_at: user.created_at,
            updated_at: user.updated_at,
            id: user.user_id,
            name: user.user_name,
            email: user.user_email,
            password_hash: user.user_password_hash,
            password_revision: user.user_password_revision,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_user"]
pub struct AuthUserInsert<'a> {
    pub user_name: &'a str,
    pub user_email: &'a str,
    pub user_password_hash: Option<&'a str>,
    pub user_password_revision: Option<i64>,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct AuthUserUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_name: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct AuthUserUpdatePassword<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_password_hash: String,
    pub user_password_revision: i64,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_service"]
#[primary_key(service_id)]
pub struct AuthService {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub service_id: i64,
    pub service_name: String,
    pub service_url: String,
}

impl From<AuthService> for core::Service {
    fn from(service: AuthService) -> Self {
        core::Service {
            created_at: service.created_at,
            updated_at: service.updated_at,
            id: service.service_id,
            name: service.service_name,
            url: service.service_url,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_service"]
pub struct AuthServiceInsert<'a> {
    pub service_name: &'a str,
    pub service_url: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "auth_service"]
pub struct AuthServiceUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub service_name: Option<&'a str>,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_key"]
#[primary_key(key_id)]
pub struct AuthKey {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub key_id: i64,
    pub key_name: String,
    pub key_value: String,
    pub service_id: Option<i64>,
    pub user_id: Option<i64>,
}

impl From<AuthKey> for core::Key {
    fn from(key: AuthKey) -> Self {
        core::Key {
            created_at: key.created_at,
            updated_at: key.updated_at,
            id: key.key_id,
            name: key.key_name,
            value: key.key_value,
            service_id: key.service_id,
            user_id: key.user_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_key"]
pub struct AuthKeyInsert<'a> {
    pub key_name: &'a str,
    pub key_value: &'a str,
    pub service_id: Option<i64>,
    pub user_id: Option<i64>,
}

#[derive(AsChangeset)]
#[table_name = "auth_key"]
pub struct AuthKeyUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub key_name: Option<&'a str>,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_csrf"]
#[primary_key(csrf_key)]
pub struct AuthCsrf {
    pub created_at: DateTime<Utc>,
    pub csrf_key: String,
    pub csrf_value: String,
    pub service_id: i64,
}

impl From<AuthCsrf> for core::Csrf {
    fn from(csrf: AuthCsrf) -> Self {
        core::Csrf {
            created_at: csrf.created_at,
            key: csrf.csrf_key,
            value: csrf.csrf_value,
            service_id: csrf.service_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_csrf"]
pub struct AuthCsrfInsert<'a> {
    pub csrf_key: &'a str,
    pub csrf_value: &'a str,
    pub service_id: i64,
}
