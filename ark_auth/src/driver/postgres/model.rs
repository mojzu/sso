use crate::core;
use crate::driver::postgres::schema::{auth_audit, auth_csrf, auth_key, auth_service, auth_user};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_service"]
#[primary_key(service_id)]
pub struct AuthService {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub service_id: String,
    pub service_is_enabled: bool,
    pub service_name: String,
    pub service_url: String,
}

impl From<AuthService> for core::Service {
    fn from(service: AuthService) -> Self {
        core::Service {
            created_at: service.created_at,
            updated_at: service.updated_at,
            id: service.service_id,
            is_enabled: service.service_is_enabled,
            name: service.service_name,
            url: service.service_url,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_service"]
pub struct AuthServiceInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub updated_at: &'a DateTime<Utc>,
    pub service_id: &'a str,
    pub service_is_enabled: bool,
    pub service_name: &'a str,
    pub service_url: &'a str,
}

#[derive(AsChangeset)]
#[table_name = "auth_service"]
pub struct AuthServiceUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub service_is_enabled: Option<bool>,
    pub service_name: Option<&'a str>,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_user"]
#[primary_key(user_id)]
pub struct AuthUser {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: String,
    pub user_is_enabled: bool,
    pub user_name: String,
    pub user_email: String,
    pub user_password_hash: Option<String>,
}

impl From<AuthUser> for core::User {
    fn from(user: AuthUser) -> Self {
        core::User {
            created_at: user.created_at,
            updated_at: user.updated_at,
            id: user.user_id,
            is_enabled: user.user_is_enabled,
            name: user.user_name,
            email: user.user_email,
            password_hash: user.user_password_hash,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_user"]
pub struct AuthUserInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub updated_at: &'a DateTime<Utc>,
    pub user_id: &'a str,
    pub user_is_enabled: bool,
    pub user_name: &'a str,
    pub user_email: &'a str,
    pub user_password_hash: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct AuthUserUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_is_enabled: Option<bool>,
    pub user_name: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct AuthUserUpdatePassword<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_password_hash: String,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_key"]
#[primary_key(key_id)]
pub struct AuthKey {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub key_id: String,
    pub key_is_enabled: bool,
    pub key_is_revoked: bool,
    pub key_name: String,
    pub key_value: String,
    pub service_id: Option<String>,
    pub user_id: Option<String>,
}

impl From<AuthKey> for core::Key {
    fn from(key: AuthKey) -> Self {
        core::Key {
            created_at: key.created_at,
            updated_at: key.updated_at,
            id: key.key_id,
            is_enabled: key.key_is_enabled,
            is_revoked: key.key_is_revoked,
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
    pub created_at: &'a DateTime<Utc>,
    pub updated_at: &'a DateTime<Utc>,
    pub key_id: &'a str,
    pub key_is_enabled: bool,
    pub key_is_revoked: bool,
    pub key_name: &'a str,
    pub key_value: &'a str,
    pub service_id: Option<&'a str>,
    pub user_id: Option<&'a str>,
}

#[derive(AsChangeset)]
#[table_name = "auth_key"]
pub struct AuthKeyUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub key_is_enabled: Option<bool>,
    pub key_is_revoked: Option<bool>,
    pub key_name: Option<&'a str>,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_csrf"]
#[primary_key(csrf_key)]
pub struct AuthCsrf {
    pub created_at: DateTime<Utc>,
    pub csrf_key: String,
    pub csrf_value: String,
    pub csrf_ttl: DateTime<Utc>,
    pub service_id: String,
}

impl From<AuthCsrf> for core::Csrf {
    fn from(csrf: AuthCsrf) -> Self {
        core::Csrf {
            created_at: csrf.created_at,
            key: csrf.csrf_key,
            value: csrf.csrf_value,
            ttl: csrf.csrf_ttl,
            service_id: csrf.service_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_csrf"]
pub struct AuthCsrfInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub csrf_key: &'a str,
    pub csrf_value: &'a str,
    pub csrf_ttl: &'a DateTime<Utc>,
    pub service_id: &'a str,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_audit"]
#[primary_key(audit_id)]
pub struct AuthAudit {
    pub created_at: DateTime<Utc>,
    pub audit_id: String,
    pub audit_user_agent: String,
    pub audit_remote: String,
    pub audit_forwarded: Option<String>,
    pub audit_path: String,
    pub audit_data: Value,
    pub key_id: Option<String>,
    pub service_id: Option<String>,
    pub user_id: Option<String>,
    pub user_key_id: Option<String>,
}

impl From<AuthAudit> for core::Audit {
    fn from(audit: AuthAudit) -> Self {
        core::Audit {
            created_at: audit.created_at,
            id: audit.audit_id,
            user_agent: audit.audit_user_agent,
            remote: audit.audit_remote,
            forwarded: audit.audit_forwarded,
            path: audit.audit_path,
            data: audit.audit_data,
            key_id: audit.key_id,
            service_id: audit.service_id,
            user_id: audit.user_id,
            user_key_id: audit.user_key_id,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "auth_audit"]
pub struct AuthAuditInsert<'a> {
    pub created_at: &'a DateTime<Utc>,
    pub audit_id: &'a str,
    pub audit_user_agent: &'a str,
    pub audit_remote: &'a str,
    pub audit_forwarded: Option<&'a str>,
    pub audit_path: &'a str,
    pub audit_data: &'a Value,
    pub key_id: Option<&'a str>,
    pub service_id: Option<&'a str>,
    pub user_id: Option<&'a str>,
    pub user_key_id: Option<&'a str>,
}
