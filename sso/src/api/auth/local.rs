use crate::{
    api::{
        csrf_verify, result_audit, validate, ApiResult, AuditIdOptResponse, AuthTokenRequest,
        ValidateRequest,
    },
    AuditBuilder, AuditMeta, AuditType, Driver, DriverError, KeyUpdate, TemplateEmail,
    UserPasswordMeta, UserToken,
};
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
    audit_meta: AuditMeta,
    key_value: Option<String>,
    password_meta: UserPasswordMeta,
    request: AuthLoginRequest,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> ApiResult<AuthLoginResponse> {
    AuthLoginRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalLogin);

    let res = provider_local::login(
        driver,
        &mut audit,
        key_value,
        request,
        access_token_expires,
        refresh_token_expires,
    );
    result_audit(driver, &audit, res).map(|data| AuthLoginResponse {
        meta: password_meta,
        data,
    })
}

pub fn auth_provider_local_register<F, E>(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthRegisterRequest,
    access_token_expires: i64,
    email: F,
) -> ApiResult<()>
where
    F: FnOnce(TemplateEmail) -> Result<(), E>,
    E: Into<DriverError>,
{
    AuthRegisterRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalRegister);

    let res = provider_local::register(
        driver,
        &mut audit,
        key_value,
        request,
        access_token_expires,
        email,
    );
    result_audit(driver, &audit, res)
        // Catch Err result so this function returns Ok to prevent the caller
        // from inferring a users existence.
        .or_else(|_e| Ok(()))
}

pub fn auth_provider_local_register_confirm(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    password_meta: UserPasswordMeta,
    request: AuthRegisterConfirmRequest,
) -> ApiResult<AuthPasswordMetaResponse> {
    AuthRegisterConfirmRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalRegisterConfirm);

    let res = provider_local::register_confirm(driver, &mut audit, key_value, request);
    result_audit(driver, &audit, res).map(|_| AuthPasswordMetaResponse {
        meta: password_meta,
    })
}

pub fn auth_provider_local_reset_password<F, E>(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthResetPasswordRequest,
    access_token_expires: i64,
    email: F,
) -> ApiResult<()>
where
    F: FnOnce(TemplateEmail) -> Result<(), E>,
    E: Into<DriverError>,
{
    AuthResetPasswordRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPassword);

    let res = provider_local::reset_password(
        driver,
        &mut audit,
        key_value,
        request,
        access_token_expires,
        email,
    );
    result_audit(driver, &audit, res)
        // Catch Err result so this function returns Ok to prevent the caller
        // from inferring a users existence.
        .or_else(|_e| Ok(()))
}

pub fn auth_provider_local_reset_password_confirm(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    password_meta: UserPasswordMeta,
    request: AuthResetPasswordConfirmRequest,
) -> ApiResult<AuthPasswordMetaResponse> {
    AuthResetPasswordConfirmRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalResetPasswordConfirm);

    let res = provider_local::reset_password_confirm(driver, &mut audit, key_value, request);
    result_audit(driver, &audit, res).map(|_| AuthPasswordMetaResponse {
        meta: password_meta,
    })
}

pub fn auth_provider_local_update_email<F, E>(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthUpdateEmailRequest,
    revoke_token_expires: i64,
    email: F,
) -> ApiResult<()>
where
    F: FnOnce(TemplateEmail) -> Result<(), E>,
    E: Into<DriverError>,
{
    AuthUpdateEmailRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdateEmail);

    let res = provider_local::update_email(
        driver,
        &mut audit,
        key_value,
        request,
        revoke_token_expires,
        email,
    );
    result_audit(driver, &audit, res)
}

pub fn auth_provider_local_update_email_revoke(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthTokenRequest,
) -> ApiResult<AuditIdOptResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdateEmailRevoke);

    let res = provider_local::update_email_revoke(driver, &mut audit, key_value, request);
    result_audit(driver, &audit, res).map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
}

pub fn auth_provider_local_update_password<F, E>(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    password_meta: UserPasswordMeta,
    request: AuthUpdatePasswordRequest,
    revoke_token_expires: i64,
    email: F,
) -> ApiResult<AuthPasswordMetaResponse>
where
    F: FnOnce(TemplateEmail) -> Result<(), E>,
    E: Into<DriverError>,
{
    AuthUpdatePasswordRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdatePassword);

    let res = provider_local::update_password(
        driver,
        &mut audit,
        key_value,
        request,
        revoke_token_expires,
        email,
    );
    result_audit(driver, &audit, res).map(|_| AuthPasswordMetaResponse {
        meta: password_meta,
    })
}

pub fn auth_provider_local_update_password_revoke(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuthTokenRequest,
) -> ApiResult<AuditIdOptResponse> {
    AuthTokenRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuthLocalUpdatePasswordRevoke);

    let res = provider_local::update_password_revoke(driver, &mut audit, key_value, request);
    result_audit(driver, &audit, res).map(|audit| AuditIdOptResponse {
        audit: audit.map(|x| x.id),
    })
}

mod provider_local {
    use super::*;
    use crate::{
        api::{ApiError, ApiResult},
        pattern::*,
        Audit, AuditBuilder, Driver, DriverError, Jwt, KeyCreate, KeyType, TemplateEmail,
        UserCreate, UserToken, UserUpdate,
    };

    pub fn login(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthLoginRequest,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> ApiResult<UserToken> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Login requires token key type.
        let user = user_read_email_checked(driver, Some(&service), audit, request.email)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Forbidden if user password update required.
        if user.password_require_update {
            return Err(ApiError::Forbidden(DriverError::UserPasswordUpdateRequired));
        }

        // Check user password.
        user.password_check(&request.password)
            .map_err(ApiError::BadRequest)?;

        // Encode user token.
        Jwt::encode_user_token(
            driver,
            &service,
            user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )
        .map_err(ApiError::BadRequest)
    }

    pub fn register<F, E>(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthRegisterRequest,
        access_token_expires: i64,
        email: F,
    ) -> ApiResult<()>
    where
        F: FnOnce(TemplateEmail) -> Result<(), E>,
        E: Into<DriverError>,
    {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Bad request if service not allowed to register users.
        if !service.user_allow_register {
            return Err(ApiError::BadRequest(
                DriverError::ServiceUserRegisterDisabled,
            ));
        }

        // Create user, is allowed to request password reset in case register token expires.
        // TODO(refactor): Support user for email already exists, add test for this.
        let mut user_create =
            UserCreate::new(true, &request.name, request.email).password_allow_reset(true);
        if let Some(locale) = request.locale {
            user_create = user_create.locale(locale);
        }
        if let Some(timezone) = request.timezone {
            user_create = user_create.timezone(timezone);
        }
        let user = driver
            .user_create(&user_create)
            .map_err(ApiError::BadRequest)?;
        // Create token key for user.
        let key_create = KeyCreate::user(true, KeyType::Token, &request.name, service.id, user.id);
        let key = driver
            .key_create(&key_create)
            .map_err(ApiError::BadRequest)?;

        // Encode register token.
        let token = Jwt::encode_register_token(driver, &service, &user, &key, access_token_expires)
            .map_err(ApiError::BadRequest)?;

        // Send register email.
        let e = TemplateEmail::email_register(&service, &user, &token, audit.meta())
            .map_err(ApiError::BadRequest)?;
        email(e)
            .map_err::<DriverError, _>(Into::into)
            .map_err(ApiError::BadRequest)?;
        Ok(())
    }

    pub fn register_confirm(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthRegisterConfirmRequest,
    ) -> ApiResult<()> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Bad request if service not allowed to register users.
        if !service.user_allow_register {
            return Err(ApiError::BadRequest(
                DriverError::ServiceUserRegisterDisabled,
            ));
        }

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Register confirm requires token key type.
        let user = user_read_id_checked(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Safely decode token with user key.
        let csrf_key = Jwt::decode_register_token(&service, &user, &key, &request.token)
            .map_err(ApiError::BadRequest)?;

        // Verify CSRF to prevent reuse.
        csrf_verify(driver, &service, &csrf_key)?;

        // Update user password and allow reset flag if provided.
        if let Some(password) = request.password {
            let mut user_update =
                UserUpdate::new_password(password).map_err(ApiError::BadRequest)?;
            if let Some(password_allow_reset) = request.password_allow_reset {
                user_update = user_update.set_password_allow_reset(password_allow_reset);
            }
            driver
                .user_update(&user.id, &user_update)
                .map_err(ApiError::BadRequest)?;
        }
        Ok(())
    }

    pub fn reset_password<F, E>(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthResetPasswordRequest,
        access_token_expires: i64,
        email: F,
    ) -> ApiResult<()>
    where
        F: FnOnce(TemplateEmail) -> Result<(), E>,
        E: Into<DriverError>,
    {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Reset password requires token key type.
        let user = user_read_email_checked(driver, Some(&service), audit, request.email)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Bad request if user password reset is disabled.
        if !user.password_allow_reset {
            return Err(ApiError::BadRequest(DriverError::UserResetPasswordDisabled));
        }

        // Encode reset token.
        let token =
            Jwt::encode_reset_password_token(driver, &service, &user, &key, access_token_expires)
                .map_err(ApiError::BadRequest)?;

        // Send reset password email.
        let e = TemplateEmail::email_reset_password(&service, &user, &token, audit.meta())
            .map_err(ApiError::BadRequest)?;
        email(e)
            .map_err::<DriverError, _>(Into::into)
            .map_err(ApiError::BadRequest)?;
        Ok(())
    }

    pub fn reset_password_confirm(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthResetPasswordConfirmRequest,
    ) -> ApiResult<()> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Reset password confirm requires token key type.
        let user = user_read_id_checked(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Bad request if user password reset is disabled.
        if !user.password_allow_reset {
            return Err(ApiError::BadRequest(DriverError::UserResetPasswordDisabled));
        }

        // Safely decode token with user key.
        let csrf_key = Jwt::decode_reset_password_token(&service, &user, &key, &request.token)
            .map_err(ApiError::BadRequest)?;

        // Verify CSRF to prevent reuse.
        csrf_verify(driver, &service, &csrf_key)?;

        // Update user password.
        let user_update =
            UserUpdate::new_password(request.password).map_err(ApiError::BadRequest)?;
        driver
            .user_update(&user.id, &user_update)
            .map_err(ApiError::BadRequest)?;
        Ok(())
    }

    pub fn update_email<F, E>(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthUpdateEmailRequest,
        revoke_token_expires: i64,
        email: F,
    ) -> ApiResult<()>
    where
        F: FnOnce(TemplateEmail) -> Result<(), E>,
        E: Into<DriverError>,
    {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Update email requires token key type.
        let user = user_read_id_checked(driver, Some(&service), audit, request.user_id)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Forbidden if user password update required.
        if user.password_require_update {
            return Err(ApiError::Forbidden(DriverError::UserPasswordUpdateRequired));
        }

        // Check user password.
        user.password_check(&request.password)
            .map_err(ApiError::BadRequest)?;

        // Encode revoke token.
        let token =
            Jwt::encode_update_email_token(driver, &service, &user, &key, revoke_token_expires)
                .map_err(ApiError::BadRequest)?;

        // Update user email.
        let old_email = user.email.to_owned();
        driver
            .user_update(&user.id, &UserUpdate::new_email(request.new_email))
            .map_err(ApiError::BadRequest)?;
        let user = user_read_id_checked(driver, Some(&service), audit, request.user_id)
            .map_err(ApiError::BadRequest)?;

        // Send update email email.
        let e =
            TemplateEmail::email_update_email(&service, &user, &old_email, &token, audit.meta())
                .map_err(ApiError::BadRequest)?;
        email(e)
            .map_err::<DriverError, _>(Into::into)
            .map_err(ApiError::BadRequest)?;
        Ok(())
    }

    pub fn update_email_revoke(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthTokenRequest,
    ) -> ApiResult<Option<Audit>> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Update email revoke requires token key type.
        // Do not check user, key is enabled or not revoked.
        let user = user_read_id_unchecked(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_unchecked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Safely decode token with user key.
        let csrf_key = Jwt::decode_update_email_token(&service, &user, &key, &request.token)
            .map_err(ApiError::BadRequest)?;

        // Verify CSRF to prevent reuse.
        csrf_verify(driver, &service, &csrf_key)?;

        // Disable user and disable and revoke all keys associated with user.
        driver
            .user_update(&user.id, &UserUpdate::default().set_is_enabled(false))
            .map_err(ApiError::BadRequest)?;
        driver
            .key_update_many(
                &user.id,
                &KeyUpdate {
                    is_enabled: Some(false),
                    is_revoked: Some(true),
                    name: None,
                },
            )
            .map_err(ApiError::BadRequest)?;

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x, None, None)
                .map_err(ApiError::BadRequest)?;
            Ok(Some(audit))
        } else {
            Ok(None)
        }
    }

    pub fn update_password<F, E>(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthUpdatePasswordRequest,
        revoke_token_expires: i64,
        email: F,
    ) -> ApiResult<()>
    where
        F: FnOnce(TemplateEmail) -> Result<(), E>,
        E: Into<DriverError>,
    {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Update password requires token key type.
        let user = user_read_id_checked(driver, Some(&service), audit, request.user_id)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // User is allowed to update password if `password_require_update` is true.

        // Check user password.
        user.password_check(&request.password)
            .map_err(ApiError::BadRequest)?;

        // Encode revoke token.
        let token =
            Jwt::encode_update_password_token(driver, &service, &user, &key, revoke_token_expires)
                .map_err(ApiError::BadRequest)?;

        // Update user password.
        let user_update =
            UserUpdate::new_password(request.new_password).map_err(ApiError::BadRequest)?;
        driver
            .user_update(&user.id, &user_update)
            .map_err(ApiError::BadRequest)?;
        let user = user_read_id_checked(driver, Some(&service), audit, request.user_id)
            .map_err(ApiError::BadRequest)?;

        // Send update password email.
        let e = TemplateEmail::email_update_password(&service, &user, &token, audit.meta())
            .map_err(ApiError::BadRequest)?;
        email(e)
            .map_err::<DriverError, _>(Into::into)
            .map_err(ApiError::BadRequest)?;
        Ok(())
    }

    pub fn update_password_revoke(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuthTokenRequest,
    ) -> ApiResult<Option<Audit>> {
        let service =
            key_service_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) =
            Jwt::decode_unsafe(&request.token, service.id).map_err(ApiError::BadRequest)?;

        // Update password revoke requires token key type.
        // Do not check user, key is enabled or not revoked.
        let user = user_read_id_unchecked(driver, Some(&service), audit, user_id)
            .map_err(ApiError::BadRequest)?;
        let key = key_read_user_unchecked(driver, &service, audit, &user, KeyType::Token)
            .map_err(ApiError::BadRequest)?;

        // Safely decode token with user key.
        let csrf_key = Jwt::decode_update_password_token(&service, &user, &key, &request.token)
            .map_err(ApiError::BadRequest)?;

        // Verify CSRF to prevent reuse.
        csrf_verify(driver, &service, &csrf_key)?;

        // Successful update password revoke, disable user and disable and revoke all keys associated with user.
        driver
            .user_update(&user.id, &UserUpdate::default().set_is_enabled(false))
            .map_err(ApiError::BadRequest)?;
        driver
            .key_update_many(
                &user.id,
                &KeyUpdate {
                    is_enabled: Some(false),
                    is_revoked: Some(true),
                    name: None,
                },
            )
            .map_err(ApiError::BadRequest)?;

        // Optionally create custom audit log.
        if let Some(x) = request.audit {
            let audit = audit
                .create(driver, x, None, None)
                .map_err(ApiError::BadRequest)?;
            Ok(Some(audit))
        } else {
            Ok(None)
        }
    }
}
