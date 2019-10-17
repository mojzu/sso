mod github;
mod local;
mod microsoft;

pub use crate::api::auth::{github::*, local::*, microsoft::*};

use crate::{
    api::{
        result_audit, validate, AuditCreate2Request, AuditIdOptResponse, ValidateRequest,
        ValidateRequestQuery,
    },
    AuditBuilder, AuditCreate2, AuditMeta, AuditType, Auth, AuthArgs, CoreResult, Csrf, Driver,
    Key, UserKey, UserToken, UserTokenAccess,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenRequest {
    #[validate(custom = "validate::token")]
    pub token: String,
    pub audit: Option<AuditCreate2Request>,
}

impl ValidateRequest<AuthTokenRequest> for AuthTokenRequest {}

impl AuthTokenRequest {
    pub fn new<S1: Into<String>>(token: S1, audit: Option<AuditCreate2Request>) -> Self {
        Self {
            token: token.into(),
            audit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenResponse {
    pub data: UserToken,
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenAccessResponse {
    pub data: UserTokenAccess,
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyRequest {
    #[validate(custom = "validate::key")]
    pub key: String,
    pub audit: Option<AuditCreate2Request>,
}

impl ValidateRequest<AuthKeyRequest> for AuthKeyRequest {}

impl AuthKeyRequest {
    pub fn new<S: Into<String>>(key: S, audit: Option<AuditCreate2Request>) -> Self {
        Self {
            key: key.into(),
            audit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyResponse {
    pub data: UserKey,
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTotpRequest {
    pub user_id: Uuid,
    #[validate(custom = "validate::totp")]
    pub totp: String,
}

impl ValidateRequest<AuthTotpRequest> for AuthTotpRequest {}

impl AuthTotpRequest {
    pub fn new<S: Into<String>>(user_id: Uuid, totp: S) -> Self {
        Self {
            user_id,
            totp: totp.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct AuthCsrfCreateRequest {
    #[validate(custom = "validate::csrf_expires_s")]
    pub expires_s: Option<i64>,
}

impl ValidateRequest<AuthCsrfCreateRequest> for AuthCsrfCreateRequest {}
impl ValidateRequestQuery<AuthCsrfCreateRequest> for AuthCsrfCreateRequest {}

impl AuthCsrfCreateRequest {
    pub fn new(expires_s: i64) -> Self {
        Self {
            expires_s: Some(expires_s),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthCsrfCreateResponse {
    pub data: Csrf,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthCsrfVerifyRequest {
    #[validate(custom = "validate::csrf_key")]
    pub key: String,
}

impl ValidateRequest<AuthCsrfVerifyRequest> for AuthCsrfVerifyRequest {}

impl AuthCsrfVerifyRequest {
    pub fn new<S: Into<String>>(key: S) -> Self {
        Self { key: key.into() }
    }
}

/// Authentication provider OAuth2 options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProviderOauth2 {
    pub client_id: String,
    pub client_secret: String,
}

impl AuthProviderOauth2 {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }
}

/// Authentication provider OAuth2 common arguments.
#[derive(Debug)]
pub struct AuthProviderOauth2Args<'a> {
    provider: Option<&'a AuthProviderOauth2>,
    user_agent: String,
    access_token_expires: i64,
    refresh_token_expires: i64,
}

impl<'a> AuthProviderOauth2Args<'a> {
    pub fn new<S1: Into<String>>(
        provider: Option<&'a AuthProviderOauth2>,
        user_agent: S1,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> Self {
        Self {
            provider,
            user_agent: user_agent.into(),
            access_token_expires,
            refresh_token_expires,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthOauth2UrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthOauth2CallbackRequest {
    #[validate(custom = "validate::token")]
    pub code: String,
    #[validate(custom = "validate::token")]
    pub state: String,
}

impl ValidateRequest<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}
impl ValidateRequestQuery<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}

pub fn auth_key_verify(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthKeyRequest,
) -> CoreResult<AuthKeyResponse> {
    AuthKeyRequest::api_validate(&request)?;

    Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthKeyVerify)
        .and_then(|(service, mut audit)| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::key_verify(
                AuthArgs::new(driver, &service, &mut audit),
                request.key,
                audit_create,
            )
        })
        .map(|(data, audit)| AuthKeyResponse {
            data,
            audit: audit.map(|x| x.id),
        })
}

pub fn auth_key_revoke(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthKeyRequest,
) -> CoreResult<AuditIdOptResponse> {
    AuthKeyRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthKeyRevoke);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::key_revoke(
                AuthArgs::new(driver, &service, &mut audit),
                request.key,
                audit_create,
            )
        }),
    )
    .map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_token_verify(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthTokenRequest,
) -> CoreResult<AuthTokenAccessResponse> {
    AuthTokenRequest::api_validate(&request)?;

    Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthTokenVerify)
        .and_then(|(service, mut audit)| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::token_verify(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
            )
        })
        .map(|(data, audit)| AuthTokenAccessResponse {
            data,
            audit: audit.map(|x| x.id),
        })
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

    Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthTokenRefresh)
        .and_then(|(service, mut audit)| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::token_refresh(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
                access_token_expires,
                refresh_token_expires,
            )
        })
        .map(|(data, audit)| AuthTokenResponse {
            data,
            audit: audit.map(|x| x.id),
        })
}

pub fn auth_token_revoke(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthTokenRequest,
) -> CoreResult<AuditIdOptResponse> {
    AuthTokenRequest::api_validate(&request)?;

    Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthTokenRevoke)
        .and_then(|(service, mut audit)| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::token_revoke(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
            )
        })
        .map(|audit| AuditIdOptResponse {
            audit: audit.map(|x| x.id),
        })
}

pub fn auth_totp(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthTotpRequest,
) -> CoreResult<()> {
    AuthTotpRequest::api_validate(&request)?;

    Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthTotp)
        .and_then(|(service, mut audit)| {
            Auth::totp(
                AuthArgs::new(driver, &service, &mut audit),
                request.user_id,
                request.totp,
            )
        })
        .map_err(Into::into)
}

pub fn auth_csrf_create(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthCsrfCreateRequest,
) -> CoreResult<AuthCsrfCreateResponse> {
    AuthCsrfCreateRequest::api_validate(&request)?;

    let (service, mut audit) =
        Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthCsrfCreate)?;
    let data = Auth::csrf_create(
        AuthArgs::new(driver, &service, &mut audit),
        request.expires_s,
    )?;
    Ok(AuthCsrfCreateResponse { data })
}

pub fn auth_csrf_verify(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthCsrfVerifyRequest,
) -> CoreResult<()> {
    AuthCsrfVerifyRequest::api_validate(&request)?;

    let (service, mut audit) =
        Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthCsrfVerify)?;
    Auth::csrf_verify(AuthArgs::new(driver, &service, &mut audit), request.key)
}
