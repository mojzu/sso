pub mod api_types;
mod oauth2;
mod validate;

pub use crate::core::api::validate::*;

use crate::{
    core::api::{api_types::*, oauth2::*},
    Audit, AuditList, AuditMeta, Auth, CoreError, CoreResult, Driver, Key, Metrics,
    ServerOptionsProviderOauth2,
};
use prometheus::Registry;
use serde_json::Value;
use uuid::Uuid;

/// API.
pub struct Api;

// TODO(refactor): Move server api, validate here.

impl Api {
    pub fn ping() -> Value {
        json!("pong")
    }

    pub fn metrics(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        registry: &Registry,
    ) -> CoreResult<String> {
        Key::authenticate(driver, audit_meta, key_value).and_then(|(service, mut audit)| {
            Metrics::read(driver, service.as_ref(), &mut audit, registry)
        })
    }

    pub fn audit_list(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuditListRequest,
    ) -> CoreResult<AuditListResponse> {
        AuditListRequest::api_validate(&request)?;
        let list: AuditList = request.into();

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Audit::list(driver, service.as_ref(), &mut audit, &list)
            })
            .map(|data| AuditListResponse {
                meta: list.into(),
                data,
            })
    }

    pub fn audit_create(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuditCreateRequest,
    ) -> CoreResult<AuditCreateResponse> {
        AuditCreateRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(move |(_, mut audit)| {
                audit
                    .set_user_id(request.user_id)
                    .set_user_key_id(request.user_key_id)
                    .create(driver, &request.type_, &request.data)
            })
            .map(|data| AuditCreateResponse { data })
    }

    pub fn audit_read(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        audit_id: Uuid,
    ) -> CoreResult<AuditReadResponse> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Audit::read(driver, service.as_ref(), &mut audit, audit_id)
            })
            .and_then(|audit| audit.ok_or_else(|| CoreError::NotFound))
            .map(|data| AuditReadResponse { data })
    }

    pub fn auth_provider_github_oauth2_url(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        // TODO(refactor): Move OAuth2 provider struct out of server module.
        provider: Option<&ServerOptionsProviderOauth2>,
        access_token_expires: i64,
    ) -> CoreResult<AuthOauth2UrlResponse> {
        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                github_oauth2_url(driver, &service, &mut audit, provider, access_token_expires)
            })
            .map(|url| AuthOauth2UrlResponse { url })
    }

    pub fn auth_provider_github_oauth2_callback(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        provider: Option<&ServerOptionsProviderOauth2>,
        user_agent: String,
        access_token_expires: i64,
        refresh_token_expires: i64,
        request: AuthOauth2CallbackRequest,
    ) -> CoreResult<AuthTokenResponse> {
        AuthOauth2CallbackRequest::api_validate(&request)?;

        let (service, mut audit) = Key::authenticate_service(driver, audit_meta, key_value)?;
        let (service_id, access_token) =
            github_oauth2_callback(driver, &service, &mut audit, provider, request)?;
        let user_email = github_api_user_email(user_agent, access_token)?;
        Auth::oauth2_login(
            driver,
            service_id,
            &mut audit,
            user_email,
            access_token_expires,
            refresh_token_expires,
        )
        .map(|(_service, data)| AuthTokenResponse { data })
    }

    pub fn auth_provider_microsoft_oauth2_url(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        provider: Option<&ServerOptionsProviderOauth2>,
        access_token_expires: i64,
    ) -> CoreResult<AuthOauth2UrlResponse> {
        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                microsoft_oauth2_url(driver, &service, &mut audit, provider, access_token_expires)
            })
            .map(|url| AuthOauth2UrlResponse { url })
    }

    pub fn auth_provider_microsoft_oauth2_callback(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        provider: Option<&ServerOptionsProviderOauth2>,
        user_agent: String,
        access_token_expires: i64,
        refresh_token_expires: i64,
        request: AuthOauth2CallbackRequest,
    ) -> CoreResult<AuthTokenResponse> {
        AuthOauth2CallbackRequest::api_validate(&request)?;

        let (service, mut audit) = Key::authenticate_service(driver, audit_meta, key_value)?;
        let (service_id, access_token) =
            microsoft_oauth2_callback(driver, &service, &mut audit, provider, request)?;
        let user_email = microsoft_api_user_email(user_agent, access_token)?;
        Auth::oauth2_login(
            driver,
            service_id,
            &mut audit,
            user_email,
            access_token_expires,
            refresh_token_expires,
        )
        .map(|(_service, data)| AuthTokenResponse { data })
    }
}
