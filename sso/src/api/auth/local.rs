use crate::{
    api::{
        result_audit, validate, ApiError, ApiResult, AuditIdOptResponse,
        AuthTokenRequest, ValidateRequest,
    },
    pattern::*,
    Audit, AuditBuilder, AuditMeta, AuditType, Driver, DriverError, Jwt, KeyCreate, KeyType,
    KeyUpdate, TemplateEmail, UserCreate, UserPasswordMeta, UserToken, UserUpdate,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthPasswordMetaResponse {
    pub meta: UserPasswordMeta,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::password")]
    pub password: String,
}

impl ValidateRequest<AuthLoginRequest> for AuthLoginRequest {}

impl AuthLoginRequest {
    pub fn new<E, P>(email: E, password: P) -> Self
    where
        E: Into<String>,
        P: Into<String>,
    {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginResponse {
    pub meta: UserPasswordMeta,
    pub data: UserToken,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthRegisterRequest {
    #[validate(custom = "validate::name")]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(custom = "validate::locale")]
    pub locale: Option<String>,

    #[validate(custom = "validate::timezone")]
    pub timezone: Option<String>,
}

impl ValidateRequest<AuthRegisterRequest> for AuthRegisterRequest {}

impl AuthRegisterRequest {
    pub fn new<N, E>(name: N, email: E) -> Self
    where
        N: Into<String>,
        E: Into<String>,
    {
        Self {
            name: name.into(),
            email: email.into(),
            locale: None,
            timezone: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthRegisterConfirmRequest {
    #[validate(custom = "validate::token")]
    pub token: String,

    #[validate(custom = "validate::password")]
    pub password: Option<String>,

    pub password_allow_reset: Option<bool>,
}

impl ValidateRequest<AuthRegisterConfirmRequest> for AuthRegisterConfirmRequest {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthResetPasswordRequest {
    #[validate(email)]
    pub email: String,
}

impl ValidateRequest<AuthResetPasswordRequest> for AuthResetPasswordRequest {}

impl AuthResetPasswordRequest {
    pub fn new<S1: Into<String>>(email: S1) -> Self {
        Self {
            email: email.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthResetPasswordConfirmRequest {
    #[validate(custom = "validate::token")]
    pub token: String,

    #[validate(custom = "validate::password")]
    pub password: String,
}

impl ValidateRequest<AuthResetPasswordConfirmRequest> for AuthResetPasswordConfirmRequest {}

impl AuthResetPasswordConfirmRequest {
    pub fn new<S1, S2>(token: S1, password: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            token: token.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdateEmailRequest {
    pub user_id: Uuid,
    #[validate(custom = "validate::password")]
    pub password: String,
    #[validate(email)]
    pub new_email: String,
}

impl ValidateRequest<AuthUpdateEmailRequest> for AuthUpdateEmailRequest {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdatePasswordRequest {
    pub user_id: Uuid,
    #[validate(custom = "validate::password")]
    pub password: String,
    #[validate(custom = "validate::password")]
    pub new_password: String,
}

impl ValidateRequest<AuthUpdatePasswordRequest> for AuthUpdatePasswordRequest {}
