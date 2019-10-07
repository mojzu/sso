pub mod api_type;
mod oauth2;
mod validate;

pub use crate::core::api::validate::*;

use crate::{
    core::api::{api_type::*, oauth2::*},
    Audit, AuditData, AuditList, AuditMeta, Auth, CoreError, CoreResult, Driver, Key, KeyList,
    Metrics, NotifyActor, Service, ServiceCreate, ServiceList, ServiceUpdate, User, UserCreate,
    UserList, UserPasswordMeta, UserRead,
};
use actix::Addr;
use prometheus::Registry;
use serde_json::Value;
use uuid::Uuid;

/// API Paths
pub mod api_path {
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
    pub const AUDIT: &str = "/audit";
    pub const SERVICE: &str = "/service";
    pub const USER: &str = "/user";
}

/// API Routes
pub mod api_route {
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

/// API provider OAuth2 options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProviderOauth2 {
    pub client_id: String,
    pub client_secret: String,
}

impl ApiProviderOauth2 {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }
}

/// API functions.
pub struct Api;

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
    ) -> CoreResult<AuditReadResponse> {
        AuditCreateRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(move |(_, mut audit)| {
                audit
                    .set_user_id(request.user_id)
                    .set_user_key_id(request.user_key_id)
                    .create(driver, &request.type_, &request.data)
            })
            .map(|data| AuditReadResponse { data })
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

    pub fn key_list(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: KeyListRequest,
    ) -> CoreResult<KeyListResponse> {
        KeyListRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let list: KeyList = request.into();
                let data = Key::list(driver, service.as_ref(), &mut audit, &list)?;
                Ok(KeyListResponse {
                    meta: list.into(),
                    data,
                })
            })
            .map_err(Into::into)
    }

    pub fn key_create(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: KeyCreateRequest,
    ) -> CoreResult<KeyReadResponse> {
        KeyCreateRequest::api_validate(&request)?;

        // If service ID is some, root key is required to create service keys.
        match request.service_id {
            Some(service_id) => {
                Key::authenticate_root(driver, audit_meta, key_value).and_then(|mut audit| {
                    match request.user_id {
                        // User ID is defined, creating user key for service.
                        Some(user_id) => Key::create_user(
                            driver,
                            &mut audit,
                            request.is_enabled,
                            request.type_,
                            request.name,
                            service_id,
                            user_id,
                        ),
                        // Creating service key.
                        None => Key::create_service(
                            driver,
                            &mut audit,
                            request.is_enabled,
                            request.name,
                            service_id,
                        ),
                    }
                })
            }
            None => Key::authenticate_service(driver, audit_meta, key_value).and_then(
                |(service, mut audit)| {
                    match request.user_id {
                        // User ID is defined, creating user key for service.
                        Some(user_id) => Key::create_user(
                            driver,
                            &mut audit,
                            request.is_enabled,
                            request.type_,
                            request.name,
                            service.id,
                            user_id,
                        ),
                        // Service cannot create service keys.
                        None => Err(CoreError::BadRequest),
                    }
                },
            ),
        }
        .map_err(Into::into)
        .map(|key| KeyReadResponse { data: key })
    }

    pub fn key_read(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        key_id: Uuid,
    ) -> CoreResult<KeyReadResponse> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Key::read_opt(driver, service.as_ref(), &mut audit, key_id)
            })
            .map_err(Into::into)
            .and_then(|key| key.ok_or_else(|| CoreError::NotFound))
            .map(|key| KeyReadResponse { data: key })
    }

    pub fn key_update(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        key_id: Uuid,
        request: KeyUpdateRequest,
    ) -> CoreResult<KeyReadResponse> {
        KeyUpdateRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Key::update(
                    driver,
                    service.as_ref(),
                    &mut audit,
                    key_id,
                    request.is_enabled,
                    None,
                    request.name,
                )
            })
            .map_err(Into::into)
            .map(|key| KeyReadResponse { data: key })
    }

    pub fn key_delete(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        key_id: Uuid,
    ) -> CoreResult<usize> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Key::delete(driver, service.as_ref(), &mut audit, key_id)
            })
            .map_err(Into::into)
    }

    pub fn service_list(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: ServiceListRequest,
    ) -> CoreResult<ServiceListResponse> {
        ServiceListRequest::api_validate(&request)?;

        Key::authenticate_root(driver, audit_meta, key_value)
            .and_then(|mut audit| {
                let list: ServiceList = request.into();
                let data = Service::list(driver, &mut audit, &list)?;
                Ok(ServiceListResponse {
                    meta: list.into(),
                    data,
                })
            })
            .map_err(Into::into)
    }

    pub fn service_create(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: ServiceCreateRequest,
    ) -> CoreResult<ServiceReadResponse> {
        ServiceCreateRequest::api_validate(&request)?;

        Key::authenticate_root(driver, audit_meta, key_value)
            .and_then(|mut audit| {
                let create: ServiceCreate = request.into();
                Service::create(driver, &mut audit, &create)
            })
            .map_err(Into::into)
            .map(|service| ServiceReadResponse { data: service })
    }

    pub fn service_read(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        service_id: Uuid,
    ) -> CoreResult<ServiceReadResponse> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Service::read_opt(driver, service.as_ref(), &mut audit, service_id)
            })
            .map_err(Into::into)
            .and_then(|service| service.ok_or_else(|| CoreError::NotFound))
            .map(|service| ServiceReadResponse { data: service })
    }

    pub fn service_update(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        service_id: Uuid,
        request: ServiceUpdateRequest,
    ) -> CoreResult<ServiceReadResponse> {
        ServiceUpdateRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let update: ServiceUpdate = request.into();
                Service::update(driver, service.as_ref(), &mut audit, service_id, &update)
            })
            .map_err(Into::into)
            .map(|service| ServiceReadResponse { data: service })
    }

    pub fn service_delete(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        service_id: Uuid,
    ) -> CoreResult<usize> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Service::delete(driver, service.as_ref(), &mut audit, service_id)
            })
            .map_err(Into::into)
    }

    pub fn user_list(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: UserListRequest,
    ) -> CoreResult<UserListResponse> {
        UserListRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let list: UserList = request.into();
                let data = User::list(driver, service.as_ref(), &mut audit, &list)?;
                Ok(UserListResponse {
                    meta: list.into(),
                    data,
                })
            })
            .map_err(Into::into)
    }

    pub fn user_create(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        password_meta: UserPasswordMeta,
        request: UserCreateRequest,
    ) -> CoreResult<UserCreateResponse> {
        UserCreateRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let mut create: UserCreate = request.into();
                User::create(driver, service.as_ref(), &mut audit, &mut create)
            })
            .map_err(Into::into)
            .map(|data| UserCreateResponse {
                meta: password_meta,
                data,
            })
    }

    pub fn user_read(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        user_id: Uuid,
    ) -> CoreResult<UserReadResponse> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let read = UserRead::Id(user_id);
                User::read_opt(driver, service.as_ref(), &mut audit, &read)
            })
            .map_err(Into::into)
            .and_then(|user| user.ok_or_else(|| CoreError::NotFound))
            .map(|user| UserReadResponse { data: user })
    }

    pub fn user_update(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        user_id: Uuid,
        request: UserUpdateRequest,
    ) -> CoreResult<UserReadResponse> {
        UserUpdateRequest::api_validate(&request)?;

        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                User::update(
                    driver,
                    service.as_ref(),
                    &mut audit,
                    user_id,
                    request.is_enabled,
                    request.name,
                    request.locale,
                    request.timezone,
                    request.password_allow_reset,
                    request.password_require_update,
                )
            })
            .map_err(Into::into)
            .map(|user| UserReadResponse { data: user })
    }

    pub fn user_delete(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        user_id: Uuid,
    ) -> CoreResult<usize> {
        Key::authenticate(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                User::delete(driver, service.as_ref(), &mut audit, user_id)
            })
            .map_err(Into::into)
    }

    pub fn auth_provider_local_login(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        password_meta: UserPasswordMeta,
        request: AuthLoginRequest,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<AuthLoginResponse> {
        AuthLoginRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Auth::login(
                    driver,
                    &service,
                    &mut audit,
                    request.email,
                    request.password,
                    access_token_expires,
                    refresh_token_expires,
                )
            })
            .map_err(Into::into)
            .map(|data| AuthLoginResponse {
                meta: password_meta,
                data,
            })
    }

    pub fn auth_provider_local_reset_password(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthResetPasswordRequest,
        notify: &Addr<NotifyActor>,
        access_token_expires: i64,
    ) -> CoreResult<()> {
        AuthResetPasswordRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Auth::reset_password(
                    driver,
                    notify,
                    &service,
                    &mut audit,
                    request.email,
                    access_token_expires,
                )
            })
            .map_err(Into::into)
    }

    pub fn auth_provider_local_reset_password_confirm(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        password_meta: UserPasswordMeta,
        request: AuthResetPasswordConfirmRequest,
    ) -> CoreResult<AuthPasswordMetaResponse> {
        AuthResetPasswordConfirmRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Auth::reset_password_confirm(
                    driver,
                    &service,
                    &mut audit,
                    request.token,
                    request.password,
                )
            })
            .map_err(Into::into)
            .map(|_| AuthPasswordMetaResponse {
                meta: password_meta,
            })
    }

    pub fn auth_provider_local_update_email(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthUpdateEmailRequest,
        notify: &Addr<NotifyActor>,
        revoke_token_expires: i64,
    ) -> CoreResult<()> {
        AuthUpdateEmailRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Auth::update_email(
                    driver,
                    notify,
                    &service,
                    &mut audit,
                    request.user_id,
                    request.password,
                    request.new_email,
                    revoke_token_expires,
                )
            })
            .map_err(Into::into)
    }

    pub fn auth_provider_local_update_email_revoke(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTokenRequest,
    ) -> CoreResult<usize> {
        AuthTokenRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::update_email_revoke(
                    driver,
                    &service,
                    &mut audit,
                    request.token,
                    audit_data.as_ref(),
                )
            })
            .map_err(Into::into)
    }

    pub fn auth_provider_local_update_password(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        password_meta: UserPasswordMeta,
        request: AuthUpdatePasswordRequest,
        notify: &Addr<NotifyActor>,
        revoke_token_expires: i64,
    ) -> CoreResult<AuthPasswordMetaResponse> {
        AuthUpdatePasswordRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Auth::update_password(
                    driver,
                    notify,
                    &service,
                    &mut audit,
                    request.user_id,
                    request.password,
                    request.new_password,
                    revoke_token_expires,
                )
            })
            .map_err(Into::into)
            .map(|_| AuthPasswordMetaResponse {
                meta: password_meta,
            })
    }

    pub fn auth_provider_local_update_password_revoke(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTokenRequest,
    ) -> CoreResult<usize> {
        AuthTokenRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::update_password_revoke(
                    driver,
                    &service,
                    &mut audit,
                    request.token,
                    audit_data.as_ref(),
                )
            })
            .map_err(Into::into)
    }

    pub fn auth_provider_github_oauth2_url(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        provider: Option<&ApiProviderOauth2>,
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
        provider: Option<&ApiProviderOauth2>,
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
        provider: Option<&ApiProviderOauth2>,
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
        provider: Option<&ApiProviderOauth2>,
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

    pub fn auth_key_verify(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthKeyRequest,
    ) -> CoreResult<AuthKeyResponse> {
        AuthKeyRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::key_verify(
                    driver,
                    &service,
                    &mut audit,
                    request.key,
                    audit_data.as_ref(),
                )
            })
            .map_err(Into::into)
            .map(|user_key| AuthKeyResponse { data: user_key })
    }

    pub fn auth_key_revoke(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthKeyRequest,
    ) -> CoreResult<usize> {
        AuthKeyRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::key_revoke(
                    driver,
                    &service,
                    &mut audit,
                    request.key,
                    audit_data.as_ref(),
                )
            })
            .map_err(Into::into)
    }

    pub fn auth_token_verify(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTokenRequest,
    ) -> CoreResult<AuthTokenAccessResponse> {
        AuthTokenRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::token_verify(
                    driver,
                    &service,
                    &mut audit,
                    request.token,
                    audit_data.as_ref(),
                )
            })
            .map_err(Into::into)
            .map(|user_token| AuthTokenAccessResponse { data: user_token })
    }

    pub fn auth_token_refresh(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTokenRequest,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<AuthTokenResponse> {
        AuthTokenRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::token_refresh(
                    driver,
                    &service,
                    &mut audit,
                    request.token,
                    audit_data.as_ref(),
                    access_token_expires,
                    refresh_token_expires,
                )
            })
            .map_err(Into::into)
            .map(|user_token| AuthTokenResponse { data: user_token })
    }

    pub fn auth_token_revoke(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTokenRequest,
    ) -> CoreResult<usize> {
        AuthTokenRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                let audit_data: Option<AuditData> = request.audit.map(|x| x.into());
                Auth::token_revoke(
                    driver,
                    &service,
                    &mut audit,
                    request.token,
                    audit_data.as_ref(),
                )
            })
            .map_err(Into::into)
    }

    pub fn auth_totp(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTotpRequest,
    ) -> CoreResult<()> {
        AuthTotpRequest::api_validate(&request)?;

        Key::authenticate_service(driver, audit_meta, key_value)
            .and_then(|(service, mut audit)| {
                Auth::totp(driver, &service, &mut audit, request.user_id, request.totp)
            })
            .map_err(Into::into)
    }
}