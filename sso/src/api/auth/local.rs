use crate::{
    api::{result_audit, validate, AuditIdOptResponse, AuthTokenRequest, ValidateRequest},
    AuditBuilder, AuditCreate2, AuditMeta, AuditType, Auth, AuthArgs, CoreResult, Driver, Key,
    NotifyActor, UserPasswordMeta, UserToken,
};
use actix::Addr;
use uuid::Uuid;
use validator::Validate;

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
    pub fn new<S1, S2>(email: S1, password: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthPasswordMetaResponse {
    pub meta: UserPasswordMeta,
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
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalLogin);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            Auth::login(
                AuthArgs::new(driver, &service, &mut audit),
                request.email,
                request.password,
                access_token_expires,
                refresh_token_expires,
            )
        }),
    )
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
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPassword);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            Auth::reset_password(
                AuthArgs::new(driver, &service, &mut audit),
                notify,
                request.email,
                access_token_expires,
            )
        }),
    )
}

pub fn auth_provider_local_reset_password_confirm(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    password_meta: UserPasswordMeta,
    request: AuthResetPasswordConfirmRequest,
) -> CoreResult<AuthPasswordMetaResponse> {
    AuthResetPasswordConfirmRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPasswordConfirm);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            Auth::reset_password_confirm(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                request.password,
            )
        }),
    )
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
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdateEmail);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            Auth::update_email(
                AuthArgs::new(driver, &service, &mut audit),
                notify,
                request.user_id,
                request.password,
                request.new_email,
                revoke_token_expires,
            )
        }),
    )
}

pub fn auth_provider_local_update_email_revoke(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthTokenRequest,
) -> CoreResult<AuditIdOptResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdateEmailRevoke);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::update_email_revoke(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
            )
        }),
    )
    .map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
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
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdatePassword);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            Auth::update_password(
                AuthArgs::new(driver, &service, &mut audit),
                notify,
                request.user_id,
                request.password,
                request.new_password,
                revoke_token_expires,
            )
        }),
    )
    .map(|_| AuthPasswordMetaResponse {
        meta: password_meta,
    })
}

pub fn auth_provider_local_update_password_revoke(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuthTokenRequest,
) -> CoreResult<AuditIdOptResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdatePasswordRevoke);

    result_audit(
        driver,
        &mut audit,
        Key::authenticate_service(driver, &mut audit, key_value).and_then(|service| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::update_password_revoke(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
            )
        }),
    )
    .map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
}
