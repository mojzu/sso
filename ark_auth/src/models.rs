use crate::schema::auth_csrf;
use crate::schema::auth_key;
use crate::schema::auth_service;
use crate::schema::auth_user;
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
    pub user_password: Option<String>,
    pub user_password_revision: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_user"]
pub struct AuthUserInsert<'a> {
    pub user_name: &'a str,
    pub user_email: &'a str,
    pub user_password: Option<&'a str>,
    pub user_password_revision: i32,
}

#[derive(AsChangeset)]
#[table_name = "auth_user"]
pub struct AuthUserUpdate<'a> {
    pub updated_at: &'a DateTime<Utc>,
    pub user_name: Option<&'a str>,
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
    pub service_id: i64,
    pub user_id: Option<i64>,
}

#[derive(Debug, Insertable)]
#[table_name = "auth_key"]
pub struct AuthKeyInsert<'a> {
    pub key_name: &'a str,
    pub key_value: &'a str,
    pub service_id: i64,
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

#[derive(Debug, Insertable)]
#[table_name = "auth_csrf"]
pub struct AuthCsrfInsert<'a> {
    pub csrf_key: &'a str,
    pub csrf_value: &'a str,
    pub service_id: i64,
}
