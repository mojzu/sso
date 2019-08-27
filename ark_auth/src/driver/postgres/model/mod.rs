mod service;
mod user;

pub use crate::driver::postgres::model::service::*;
pub use crate::driver::postgres::model::user::*;

use crate::core;
use crate::driver::postgres::schema::{auth_audit, auth_csrf, auth_key};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_key"]
#[primary_key(key_id)]
pub struct AuthKey {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub key_id: Uuid,
    pub key_is_enabled: bool,
    pub key_is_revoked: bool,
    pub key_name: String,
    pub key_value: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
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
    pub key_id: Uuid,
    pub key_is_enabled: bool,
    pub key_is_revoked: bool,
    pub key_name: &'a str,
    pub key_value: &'a str,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
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
    pub service_id: Uuid,
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
    pub service_id: Uuid,
}

#[derive(Debug, Identifiable, Queryable)]
#[table_name = "auth_audit"]
#[primary_key(audit_id)]
pub struct AuthAudit {
    pub created_at: DateTime<Utc>,
    pub audit_id: Uuid,
    pub audit_user_agent: String,
    pub audit_remote: String,
    pub audit_forwarded: Option<String>,
    pub audit_path: String,
    pub audit_data: Value,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
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
    pub audit_id: Uuid,
    pub audit_user_agent: &'a str,
    pub audit_remote: &'a str,
    pub audit_forwarded: Option<&'a str>,
    pub audit_path: &'a str,
    pub audit_data: &'a Value,
    pub key_id: Option<Uuid>,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}
