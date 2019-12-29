mod local;

pub use crate::api::auth::{github::*, local::*, microsoft::*};

use crate::{
    api::{
        result_audit, result_audit_err, validate, ApiResult, AuditIdOptResponse, ValidateRequest,
        ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Csrf, Driver, KeyUpdate, UserKey, UserToken,
    UserTokenAccess,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenRequest {
    #[validate(custom = "validate::token")]
    pub token: String,
    pub audit: Option<String>,
}

impl ValidateRequest<AuthTokenRequest> for AuthTokenRequest {}

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
    pub audit: Option<String>,
}

impl ValidateRequest<AuthKeyRequest> for AuthKeyRequest {}

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
