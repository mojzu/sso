use crate::{
    api::{
        oauth2::*, Api, ApiProviderOauth2Args, ApiValidate, ApiValidateRequest,
        ApiValidateRequestQuery, AuditCreate2Request, AuditIdOptResponse,
    },
    AuditCreate2, AuditMeta, AuditType, Auth, AuthArgs, CoreResult, Csrf, Driver, Key, NotifyActor,
    UserKey, UserPasswordMeta, UserToken, UserTokenAccess,
};
use actix::Addr;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenRequest {
    #[validate(custom = "ApiValidate::token")]
    pub token: String,
    pub audit: Option<AuditCreate2Request>,
}

impl ApiValidateRequest<AuthTokenRequest> for AuthTokenRequest {}

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
    #[validate(custom = "ApiValidate::key")]
    pub key: String,
    pub audit: Option<AuditCreate2Request>,
}

impl ApiValidateRequest<AuthKeyRequest> for AuthKeyRequest {}

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
    #[validate(custom = "ApiValidate::totp")]
    pub totp: String,
}

impl ApiValidateRequest<AuthTotpRequest> for AuthTotpRequest {}

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
    #[validate(custom = "ApiValidate::csrf_expires_s")]
    pub expires_s: Option<i64>,
}

impl ApiValidateRequest<AuthCsrfCreateRequest> for AuthCsrfCreateRequest {}
impl ApiValidateRequestQuery<AuthCsrfCreateRequest> for AuthCsrfCreateRequest {}

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
    #[validate(custom = "ApiValidate::csrf_key")]
    pub key: String,
}

impl ApiValidateRequest<AuthCsrfVerifyRequest> for AuthCsrfVerifyRequest {}

impl AuthCsrfVerifyRequest {
    pub fn new<S: Into<String>>(key: S) -> Self {
        Self { key: key.into() }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
}

impl ApiValidateRequest<AuthLoginRequest> for AuthLoginRequest {}

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

impl ApiValidateRequest<AuthResetPasswordRequest> for AuthResetPasswordRequest {}

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
    #[validate(custom = "ApiValidate::token")]
    pub token: String,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
}

impl ApiValidateRequest<AuthResetPasswordConfirmRequest> for AuthResetPasswordConfirmRequest {}

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
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
    #[validate(email)]
    pub new_email: String,
}

impl ApiValidateRequest<AuthUpdateEmailRequest> for AuthUpdateEmailRequest {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdatePasswordRequest {
    pub user_id: Uuid,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
    #[validate(custom = "ApiValidate::password")]
    pub new_password: String,
}

impl ApiValidateRequest<AuthUpdatePasswordRequest> for AuthUpdatePasswordRequest {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthOauth2UrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthOauth2CallbackRequest {
    #[validate(custom = "ApiValidate::token")]
    pub code: String,
    #[validate(custom = "ApiValidate::token")]
    pub state: String,
}

impl ApiValidateRequest<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}
impl ApiValidateRequestQuery<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}

impl Api {
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

        Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthLocalLogin)
            .and_then(|(service, mut audit)| {
                Auth::login(
                    AuthArgs::new(driver, &service, &mut audit),
                    request.email,
                    request.password,
                    access_token_expires,
                    refresh_token_expires,
                )
            })
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

        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthLocalResetPassword,
        )
        .and_then(|(service, mut audit)| {
            Auth::reset_password(
                AuthArgs::new(driver, &service, &mut audit),
                notify,
                request.email,
                access_token_expires,
            )
        })
    }

    pub fn auth_provider_local_reset_password_confirm(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        password_meta: UserPasswordMeta,
        request: AuthResetPasswordConfirmRequest,
    ) -> CoreResult<AuthPasswordMetaResponse> {
        AuthResetPasswordConfirmRequest::api_validate(&request)?;

        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthLocalResetPasswordConfirm,
        )
        .and_then(|(service, mut audit)| {
            Auth::reset_password_confirm(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                request.password,
            )
        })
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

        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthLocalUpdateEmail,
        )
        .and_then(|(service, mut audit)| {
            Auth::update_email(
                AuthArgs::new(driver, &service, &mut audit),
                notify,
                request.user_id,
                request.password,
                request.new_email,
                revoke_token_expires,
            )
        })
    }

    pub fn auth_provider_local_update_email_revoke(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        request: AuthTokenRequest,
    ) -> CoreResult<AuditIdOptResponse> {
        AuthTokenRequest::api_validate(&request)?;

        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthLocalUpdateEmailRevoke,
        )
        .and_then(|(service, mut audit)| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::update_email_revoke(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
            )
        })
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

        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthLocalUpdatePassword,
        )
        .and_then(|(service, mut audit)| {
            Auth::update_password(
                AuthArgs::new(driver, &service, &mut audit),
                notify,
                request.user_id,
                request.password,
                request.new_password,
                revoke_token_expires,
            )
        })
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

        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthLocalUpdatePasswordRevoke,
        )
        .and_then(|(service, mut audit)| {
            let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
            Auth::update_password_revoke(
                AuthArgs::new(driver, &service, &mut audit),
                request.token,
                audit_create,
            )
        })
        .map(|audit| AuditIdOptResponse {
            audit: audit.map(|x| x.id),
        })
    }

    pub fn auth_provider_github_oauth2_url(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        args: ApiProviderOauth2Args,
    ) -> CoreResult<AuthOauth2UrlResponse> {
        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthGithubOauth2Url,
        )
        .and_then(|(service, mut audit)| {
            github_oauth2_url(
                driver,
                &service,
                &mut audit,
                args.provider,
                args.access_token_expires,
            )
        })
        .map(|url| AuthOauth2UrlResponse { url })
    }

    pub fn auth_provider_github_oauth2_callback(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        args: ApiProviderOauth2Args,
        request: AuthOauth2CallbackRequest,
    ) -> CoreResult<AuthTokenResponse> {
        AuthOauth2CallbackRequest::api_validate(&request)?;

        let (service, mut audit) = Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthGithubOauth2Callback,
        )?;
        let (service_id, access_token) =
            github_oauth2_callback(driver, &service, &mut audit, args.provider, request)?;
        let user_email = github_api_user_email(args.user_agent, access_token)?;
        Auth::oauth2_login(
            AuthArgs::new(driver, &service, &mut audit),
            service_id,
            user_email,
            args.access_token_expires,
            args.refresh_token_expires,
        )
        .map(|(_service, data)| AuthTokenResponse { data, audit: None })
    }

    pub fn auth_provider_microsoft_oauth2_url(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        args: ApiProviderOauth2Args,
    ) -> CoreResult<AuthOauth2UrlResponse> {
        Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthMicrosoftOauth2Url,
        )
        .and_then(|(service, mut audit)| {
            microsoft_oauth2_url(
                driver,
                &service,
                &mut audit,
                args.provider,
                args.access_token_expires,
            )
        })
        .map(|url| AuthOauth2UrlResponse { url })
    }

    pub fn auth_provider_microsoft_oauth2_callback(
        driver: &dyn Driver,
        key_value: Option<String>,
        audit_meta: AuditMeta,
        args: ApiProviderOauth2Args,
        request: AuthOauth2CallbackRequest,
    ) -> CoreResult<AuthTokenResponse> {
        AuthOauth2CallbackRequest::api_validate(&request)?;

        let (service, mut audit) = Key::authenticate_service(
            driver,
            audit_meta,
            key_value,
            AuditType::AuthMicrosoftOauth2Callback,
        )?;
        let (service_id, access_token) =
            microsoft_oauth2_callback(driver, &service, &mut audit, args.provider, request)?;
        let user_email = microsoft_api_user_email(args.user_agent, access_token)?;
        Auth::oauth2_login(
            AuthArgs::new(driver, &service, &mut audit),
            service_id,
            user_email,
            args.access_token_expires,
            args.refresh_token_expires,
        )
        .map(|(_service, data)| AuthTokenResponse { data, audit: None })
    }

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

        Key::authenticate_service(driver, audit_meta, key_value, AuditType::AuthKeyRevoke)
            .and_then(|(service, mut audit)| {
                let audit_create: Option<AuditCreate2> = request.audit.map(|x| x.into());
                Auth::key_revoke(
                    AuthArgs::new(driver, &service, &mut audit),
                    request.key,
                    audit_create,
                )
            })
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
}
