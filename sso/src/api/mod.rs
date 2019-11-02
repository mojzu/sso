//! # API Functions
mod audit;
mod auth;
mod error;
mod key;
mod password;
mod service;
mod user;
pub mod validate;

pub use crate::api::{
    audit::*,
    auth::*,
    error::*,
    key::*,
    password::*,
    service::*,
    user::*,
    validate::{ValidateRequest, ValidateRequestQuery},
};

use crate::{
    AuditBuilder, AuditDiff, AuditDiffBuilder, AuditMeta, AuditSubject, AuditType, Driver,
    DriverError, Service,
};
use http::StatusCode;
use serde_json::Value;

/// API Paths
pub mod path {
    pub const NONE: &str = "";
    pub const ID: &str = "/{id}";
    pub const V1: &str = "/v1";
    pub const PING: &str = "/ping";
    pub const METRICS: &str = "/metrics";
    pub const AUTH: &str = "/auth";
    pub const PROVIDER: &str = "/provider";
    pub const LOCAL: &str = "/local";
    pub const LOGIN: &str = "/login";
    pub const REGISTER: &str = "/register";
    pub const RESET_PASSWORD: &str = "/reset-password";
    pub const UPDATE_EMAIL: &str = "/update-email";
    pub const UPDATE_PASSWORD: &str = "/update-password";
    pub const CONFIRM: &str = "/confirm";
    pub const GITHUB: &str = "/github";
    pub const MICROSOFT: &str = "/microsoft";
    pub const OAUTH2: &str = "/oauth2";
    pub const KEY: &str = "/key";
    pub const TOKEN: &str = "/token";
    pub const VERIFY: &str = "/verify";
    pub const REFRESH: &str = "/refresh";
    pub const REVOKE: &str = "/revoke";
    pub const TOTP: &str = "/totp";
    pub const CSRF: &str = "/csrf";
    pub const AUDIT: &str = "/audit";
    pub const SERVICE: &str = "/service";
    pub const USER: &str = "/user";
}

/// API Routes
pub mod route {
    use std::fmt::Display;

    pub const PING: &str = "/v1/ping";
    pub const METRICS: &str = "/v1/metrics";
    pub const AUTH_LOCAL_LOGIN: &str = "/v1/auth/provider/local/login";
    pub const AUTH_LOCAL_REGISTER: &str = "/v1/auth/provider/local/register";
    pub const AUTH_LOCAL_REGISTER_CONFIRM: &str = "/v1/auth/provider/local/register/confirm";
    pub const AUTH_LOCAL_RESET_PASSWORD: &str = "/v1/auth/provider/local/reset-password";
    pub const AUTH_LOCAL_RESET_PASSWORD_CONFIRM: &str =
        "/v1/auth/provider/local/reset-password/confirm";
    pub const AUTH_LOCAL_UPDATE_EMAIL: &str = "/v1/auth/provider/local/update-email";
    pub const AUTH_LOCAL_UPDATE_EMAIL_REVOKE: &str = "/v1/auth/provider/local/update-email/revoke";
    pub const AUTH_LOCAL_UPDATE_PASSWORD: &str = "/v1/auth/provider/local/update-password";
    pub const AUTH_LOCAL_UPDATE_PASSWORD_REVOKE: &str =
        "/v1/auth/provider/local/update-password/revoke";
    pub const AUTH_GITHUB_OAUTH2: &str = "/v1/auth/provider/github/oauth2";
    pub const AUTH_MICROSOFT_OAUTH2: &str = "/v1/auth/provider/microsoft/oauth2";
    pub const AUTH_KEY_VERIFY: &str = "/v1/auth/key/verify";
    pub const AUTH_KEY_REVOKE: &str = "/v1/auth/key/revoke";
    pub const AUTH_TOKEN_VERIFY: &str = "/v1/auth/token/verify";
    pub const AUTH_TOKEN_REFRESH: &str = "/v1/auth/token/refresh";
    pub const AUTH_TOKEN_REVOKE: &str = "/v1/auth/token/revoke";
    pub const AUTH_TOTP: &str = "/v1/auth/totp";
    pub const AUTH_CSRF: &str = "/v1/auth/csrf";
    pub const AUDIT: &str = "/v1/audit";
    pub const KEY: &str = "/v1/key";
    pub const SERVICE: &str = "/v1/service";
    pub const USER: &str = "/v1/user";

    pub fn audit_id<T: Display>(id: T) -> String {
        format!("{}/{}", AUDIT, id)
    }

    pub fn key_id<T: Display>(id: T) -> String {
        format!("{}/{}", KEY, id)
    }

    pub fn service_id<T: Display>(id: T) -> String {
        format!("{}/{}", SERVICE, id)
    }

    pub fn user_id<T: Display>(id: T) -> String {
        format!("{}/{}", USER, id)
    }
}

pub fn server_ping() -> Value {
    json!("pong")
}

pub fn server_metrics(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
) -> ApiResult<String> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::Metrics);

    let res = server::metrics(driver, &mut audit, key_value);
    result_audit_err(driver, &audit, res)
}

mod server {
    use crate::{
        api::{ApiError, ApiResult},
        pattern::*,
        AuditBuilder, Driver, Metrics,
    };

    pub fn metrics(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> ApiResult<String> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        Metrics::read(driver, service.as_ref()).map_err(ApiError::BadRequest)
    }
}

// TODO(refactor): Unwrap check and cleanup.

fn result_audit<T>(driver: &dyn Driver, audit: &AuditBuilder, res: ApiResult<T>) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", &e);
        audit
            .create_data(driver, e.status_code(), None, Some(data))
            .unwrap();
        Err(e)
    })
    .and_then(|res| {
        audit
            .create_data::<bool>(driver, StatusCode::OK.as_u16(), None, None)
            .unwrap();
        Ok(res)
    })
}

fn result_audit_err<T>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: ApiResult<T>,
) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", &e);
        audit
            .create_data(driver, e.status_code(), None, Some(data))
            .unwrap();
        Err(e)
    })
}

fn result_audit_subject<T: AuditSubject>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: ApiResult<T>,
) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", &e);
        audit
            .create_data(driver, e.status_code(), None, Some(data))
            .unwrap();
        Err(e)
    })
    .and_then(|res| {
        audit
            .create_data::<bool>(driver, StatusCode::OK.as_u16(), Some(res.subject()), None)
            .unwrap();
        Ok(res)
    })
}

fn result_audit_diff<T: AuditSubject + AuditDiff>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: ApiResult<(T, T)>,
) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", &e);
        audit
            .create_data(driver, e.status_code(), None, Some(data))
            .unwrap();
        Err(e)
    })
    .and_then(|(p, n)| {
        let diff = n.diff(&p);
        audit
            .create_data(
                driver,
                StatusCode::OK.as_u16(),
                Some(n.subject()),
                Some(diff),
            )
            .unwrap();
        Ok(n)
    })
}

fn csrf_verify(driver: &dyn Driver, service: &Service, csrf_key: &str) -> ApiResult<()> {
    driver
        .csrf_read(&csrf_key)
        .map_err(ApiError::BadRequest)?
        .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
        .and_then(|csrf| csrf.check_service(service.id))
        .map_err(ApiError::BadRequest)
}
