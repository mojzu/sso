//! # API Module
mod audit;
mod auth;
mod error;
mod key;
mod service;
mod user;
pub mod validate;

pub use crate::api::{
    audit::*,
    auth::*,
    error::*,
    key::*,
    service::*,
    user::*,
    validate::{ValidateRequest, ValidateRequestQuery},
};

use crate::{
    AuditBuilder, AuditDiff, AuditDiffBuilder, AuditMeta, AuditSubject, AuditType, Driver,
};
use prometheus::Registry;
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
    registry: &Registry,
) -> ApiResult<String> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::Metrics);

    let res = server::metrics(driver, &mut audit, key_value, registry);
    result_audit_err(driver, &audit, res)
}

mod server {
    use crate::{
        api::{ApiError, ApiResult},
        AuditBuilder, Auth, Driver, Metrics,
    };
    use prometheus::Registry;

    pub fn metrics(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        registry: &Registry,
    ) -> ApiResult<String> {
        let service =
            Auth::authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        Metrics::read(driver, service.as_ref(), registry).map_err(ApiError::BadRequest)
    }
}

fn result_audit<T>(driver: &dyn Driver, audit: &AuditBuilder, res: ApiResult<T>) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", &e);
        audit.create_data(driver, None, Some(data)).unwrap();
        Err(e)
    })
    .and_then(|res| {
        audit.create_data::<bool>(driver, None, None).unwrap();
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
        audit.create_data(driver, None, Some(data)).unwrap();
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
        audit.create_data(driver, None, Some(data)).unwrap();
        Err(e)
    })
    .and_then(|res| {
        audit
            .create_data::<bool>(driver, Some(res.subject()), None)
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
        audit.create_data(driver, None, Some(data)).unwrap();
        Err(e)
    })
    .and_then(|(p, n)| {
        let diff = n.diff(&p);
        audit
            .create_data(driver, Some(n.subject()), Some(diff))
            .unwrap();
        Ok(n)
    })
}
