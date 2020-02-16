use crate::{
    grpc::{method::auth::api_csrf_verify, pb, util::*, validate, Server},
    *,
};
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

impl Validate for pb::AuthLoginRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::email(e, "email", &self.email);
            validate::password(e, "password", &self.password);
        })
    }
}

pub async fn login(
    server: &Server,
    request: MethodRequest<pb::AuthLoginRequest>,
) -> MethodResult<pb::AuthLoginReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let client = server.client();
    let pwned_passwords = server.options().pwned_passwords_enabled();
    let password_meta =
        pattern::password_meta(client.as_ref(), pwned_passwords, Some(req.password.clone()))
            .await
            .map_err(MethodError::BadRequest)?;

    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let refresh_token_expires = server.options().refresh_token_expires();
    method_blocking(move || {
        let user_token = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalLogin,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;

                // Login requires token key type.
                let user =
                    pattern::user_read_email_checked(driver, Some(&service), audit, &req.email)
                        .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;

                // Forbidden if user password update required.
                if user.password_require_update {
                    return Err(MethodError::Forbidden(
                        DriverError::UserPasswordUpdateRequired,
                    ));
                }

                // Check user password.
                user.password_check(&req.password)
                    .map_err(MethodError::BadRequest)?;

                // Encode user token.
                Jwt::encode_user_token(
                    driver,
                    &service,
                    user,
                    &key,
                    access_token_expires,
                    refresh_token_expires,
                )
                .map_err(MethodError::BadRequest)
            },
        )?;
        Ok((password_meta, user_token))
    })
    .await
    .map(|(password_meta, user_token)| pb::AuthLoginReply {
        meta: Some(password_meta.into()),
        user: Some(user_token.user.clone().into()),
        access: Some(user_token.access_token()),
        refresh: Some(user_token.refresh_token()),
    })
}

impl Validate for pb::AuthRegisterRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::name(e, "name", &self.name);
            validate::email(e, "email", &self.email);
            validate::locale_opt(e, "locale", self.locale.as_ref().map(|x| &**x));
            validate::timezone_opt(e, "timezone", self.timezone.as_ref().map(|x| &**x));
        })
    }
}

pub async fn register(
    server: &Server,
    request: MethodRequest<pb::AuthRegisterRequest>,
) -> MethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let email = server.smtp_email();

    method_blocking(move || {
        let template = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalRegister,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;
                // Bad request if service not allowed to register users.
                if !service.user_allow_register {
                    return Err(MethodError::BadRequest(
                        DriverError::ServiceUserRegisterDisabled,
                    ));
                }

                // Get user by email if exists, else create now.
                let user = driver
                    .user_read(&UserRead::Email(req.email.to_owned()))
                    .map_err(MethodError::BadRequest)?;
                let user = match user {
                    Some(user) => {
                        // If user is disabled, reject this request.
                        if !user.is_enabled {
                            return Err(MethodError::BadRequest(DriverError::UserDisabled));
                        }
                        user
                    }
                    None => {
                        // Create user, is allowed to request password reset in case register token expires.
                        let mut user_create =
                            UserCreate::new(true, &req.name, &req.email).password_allow_reset(true);
                        if let Some(locale) = &req.locale {
                            user_create = user_create.locale(locale);
                        }
                        if let Some(timezone) = &req.timezone {
                            user_create = user_create.timezone(timezone);
                        }
                        driver
                            .user_create(&user_create)
                            .map_err(MethodError::BadRequest)?
                    }
                };
                // Get key if exists, else create now.
                let key = driver
                    .key_read(
                        &KeyRead::user_id(service.id, user.id, true, false, KeyType::Token),
                        None,
                    )
                    .map_err(MethodError::BadRequest)?;
                let key = match key {
                    Some(key) => key,
                    None => {
                        // Create token key for user.
                        let key_create =
                            KeyCreate::user(true, KeyType::Token, &req.name, service.id, user.id);
                        driver
                            .key_create(&key_create)
                            .map_err(MethodError::BadRequest)?
                    }
                };
                // Encode register token.
                let token =
                    Jwt::encode_register_token(driver, &service, &user, &key, access_token_expires)
                        .map_err(MethodError::BadRequest)?;
                // Send register email.
                TemplateEmail::email_register(&service, &user, &token, audit.meta())
                    .map_err(MethodError::BadRequest)
            },
        )?;
        email(template)
            .map_err::<DriverError, _>(Into::into)
            .map_err(MethodError::BadRequest)?;
        Ok(())
    })
    .await
}

impl Validate for pb::AuthRegisterConfirmRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::token(e, "token", &self.token);
            validate::password_opt(e, "password", self.password.as_ref().map(|x| &**x));
        })
    }
}

pub async fn register_confirm(
    server: &Server,
    request: MethodRequest<pb::AuthRegisterConfirmRequest>,
) -> MethodResult<pb::AuthPasswordMetaReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let client = server.client();
    let pwned_passwords = server.options().pwned_passwords_enabled();
    let password_meta =
        pattern::password_meta(client.as_ref(), pwned_passwords, req.password.clone())
            .await
            .map_err(MethodError::BadRequest)?;

    let driver = server.driver();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    method_blocking(move || {
        let template = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalRegisterConfirm,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;
                // Bad request if service not allowed to register users.
                if !service.user_allow_register {
                    return Err(MethodError::BadRequest(
                        DriverError::ServiceUserRegisterDisabled,
                    ));
                }
                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) =
                    Jwt::decode_unsafe(&req.token, service.id).map_err(MethodError::BadRequest)?;
                // Register confirm requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;
                // Safely decode token with user key.
                let csrf_key = Jwt::decode_register_token(&service, &user, &key, &req.token)
                    .map_err(MethodError::BadRequest)?;
                // Verify CSRF to prevent reuse.
                api_csrf_verify(driver, &service, &csrf_key)?;
                // Encode revoke token.
                let token =
                    Jwt::encode_revoke_token(driver, &service, &user, &key, revoke_token_expires)
                        .map_err(MethodError::BadRequest)?;
                // Update user password and allow reset flag if provided.
                if let Some(password) = &req.password {
                    let mut user_update = UserUpdate::new_password(user.id, password)
                        .map_err(MethodError::BadRequest)?;
                    if let Some(password_allow_reset) = req.password_allow_reset {
                        user_update = user_update.set_password_allow_reset(password_allow_reset);
                    }
                    driver
                        .user_update(&user_update)
                        .map_err(MethodError::BadRequest)?;
                }
                // Send reset password confirm email.
                TemplateEmail::email_register_confirm(&service, &user, &token, audit.meta())
                    .map_err(MethodError::BadRequest)
            },
        )?;
        email(template)
            .map_err::<DriverError, _>(Into::into)
            .map_err(MethodError::BadRequest)?;
        Ok(password_meta)
    })
    .await
    .map(|password_meta| pb::AuthPasswordMetaReply {
        meta: Some(password_meta.into()),
    })
}

pub async fn register_revoke(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    method_blocking(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalRegisterRevoke,
            |driver, audit| revoke_inner(driver, audit, &auth, &req),
        )
        .map_err(Into::into)
    })
    .await
    .map(|audit| pb::AuthAuditReply {
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

impl Validate for pb::AuthResetPasswordRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::email(e, "email", &self.email);
        })
    }
}

pub async fn reset_password(
    server: &Server,
    request: MethodRequest<pb::AuthResetPasswordRequest>,
) -> MethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();
    let driver = server.driver();
    let access_token_expires = server.options().access_token_expires();
    let email = server.smtp_email();

    method_blocking(move || {
        let template = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalResetPassword,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;
                // Reset password requires token key type.
                let user =
                    pattern::user_read_email_checked(driver, Some(&service), audit, &req.email)
                        .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;
                // Bad request if user password reset is disabled.
                if !user.password_allow_reset {
                    return Err(MethodError::BadRequest(
                        DriverError::UserResetPasswordDisabled,
                    ));
                }
                // Encode reset token.
                let token = Jwt::encode_reset_password_token(
                    driver,
                    &service,
                    &user,
                    &key,
                    access_token_expires,
                )
                .map_err(MethodError::BadRequest)?;
                // Send reset password email.
                TemplateEmail::email_reset_password(&service, &user, &token, audit.meta())
                    .map_err(MethodError::BadRequest)
            },
        );
        // Catch Err result so this function returns Ok to prevent the caller
        // from inferring a users existence.
        match template {
            Ok(template) => email(template)
                .map_err::<DriverError, _>(Into::into)
                .map_err(MethodError::BadRequest)
                .or_else(|_| Ok(())),
            Err(_e) => Ok(()),
        }
    })
    .await
}

impl Validate for pb::AuthResetPasswordConfirmRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::token(e, "token", &self.token);
            validate::password(e, "password", &self.password);
        })
    }
}

pub async fn reset_password_confirm(
    server: &Server,
    request: MethodRequest<pb::AuthResetPasswordConfirmRequest>,
) -> MethodResult<pb::AuthPasswordMetaReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let client = server.client();
    let pwned_passwords = server.options().pwned_passwords_enabled();
    let password_meta =
        pattern::password_meta(client.as_ref(), pwned_passwords, Some(req.password.clone()))
            .await
            .map_err(MethodError::BadRequest)?;

    let driver = server.driver();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    method_blocking(move || {
        let template = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalResetPasswordConfirm,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;

                // Unsafely decode token to get user identifier, used to read key for safe token decode.
                let (user_id, _) =
                    Jwt::decode_unsafe(&req.token, service.id).map_err(MethodError::BadRequest)?;

                // Reset password confirm requires token key type.
                let user = pattern::user_read_id_checked(driver, Some(&service), audit, user_id)
                    .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;

                // Bad request if user password reset is disabled.
                if !user.password_allow_reset {
                    return Err(MethodError::BadRequest(
                        DriverError::UserResetPasswordDisabled,
                    ));
                }

                // Safely decode token with user key.
                let csrf_key = Jwt::decode_reset_password_token(&service, &user, &key, &req.token)
                    .map_err(MethodError::BadRequest)?;

                // Verify CSRF to prevent reuse.
                api_csrf_verify(driver, &service, &csrf_key)?;

                // Encode revoke token.
                let token =
                    Jwt::encode_revoke_token(driver, &service, &user, &key, revoke_token_expires)
                        .map_err(MethodError::BadRequest)?;

                // Update user password.
                let user_update = UserUpdate::new_password(user.id, &req.password)
                    .map_err(MethodError::BadRequest)?;
                driver
                    .user_update(&user_update)
                    .map_err(MethodError::BadRequest)?;

                // Send reset password confirm email.
                TemplateEmail::email_reset_password_confirm(&service, &user, &token, audit.meta())
                    .map_err(MethodError::BadRequest)
            },
        )?;
        email(template)
            .map_err::<DriverError, _>(Into::into)
            .map_err(MethodError::BadRequest)?;
        Ok(password_meta)
    })
    .await
    .map(|password_meta| pb::AuthPasswordMetaReply {
        meta: Some(password_meta.into()),
    })
}

pub async fn reset_password_revoke(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    method_blocking(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalResetPasswordRevoke,
            |driver, audit| revoke_inner(driver, audit, &auth, &req),
        )
        .map_err(Into::into)
    })
    .await
    .map(|audit| pb::AuthAuditReply {
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

impl Validate for pb::AuthUpdateEmailRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::email(e, "email", &self.email);
            validate::password(e, "password", &self.password);
            validate::email(e, "new_email", &self.new_email);
        })
    }
}

pub async fn update_email(
    server: &Server,
    request: MethodRequest<pb::AuthUpdateEmailRequest>,
) -> MethodResult<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    method_blocking(move || {
        let template = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalUpdateEmail,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;
                // Update email requires token key type.
                let user =
                    pattern::user_read_email_checked(driver, Some(&service), audit, &req.email)
                        .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;
                // Forbidden if user password update required.
                if user.password_require_update {
                    return Err(MethodError::Forbidden(
                        DriverError::UserPasswordUpdateRequired,
                    ));
                }
                // Check user password.
                user.password_check(&req.password)
                    .map_err(MethodError::BadRequest)?;
                // Encode revoke token.
                let token =
                    Jwt::encode_revoke_token(driver, &service, &user, &key, revoke_token_expires)
                        .map_err(MethodError::BadRequest)?;
                // Update user email.
                let old_email = user.email.to_owned();
                driver
                    .user_update(&UserUpdate::new_email(user.id, &req.new_email))
                    .map_err(MethodError::BadRequest)?;
                let user =
                    pattern::user_read_email_checked(driver, Some(&service), audit, &req.new_email)
                        .map_err(MethodError::BadRequest)?;
                // Send update email email.
                TemplateEmail::email_update_email(&service, &user, &old_email, &token, audit.meta())
                    .map_err(MethodError::BadRequest)
            },
        )?;
        email(template)
            .map_err::<DriverError, _>(Into::into)
            .map_err(MethodError::BadRequest)?;
        Ok(())
    })
    .await
}

pub async fn update_email_revoke(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    method_blocking(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalUpdateEmailRevoke,
            |driver, audit| revoke_inner(driver, audit, &auth, &req),
        )
        .map_err(Into::into)
    })
    .await
    .map(|audit| pb::AuthAuditReply {
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

impl Validate for pb::AuthUpdatePasswordRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::email(e, "email", &self.email);
            validate::password(e, "password", &self.password);
            validate::password(e, "new_password", &self.new_password);
        })
    }
}

pub async fn update_password(
    server: &Server,
    request: MethodRequest<pb::AuthUpdatePasswordRequest>,
) -> MethodResult<pb::AuthPasswordMetaReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let client = server.client();
    let pwned_passwords = server.options().pwned_passwords_enabled();
    let password_meta = pattern::password_meta(
        client.as_ref(),
        pwned_passwords,
        Some(req.new_password.clone()),
    )
    .await
    .map_err(MethodError::BadRequest)?;

    let driver = server.driver();
    let revoke_token_expires = server.options().revoke_token_expires();
    let email = server.smtp_email();
    method_blocking(move || {
        let template = audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalUpdatePassword,
            |driver, audit| {
                let service = pattern::key_service_authenticate(driver, audit, &auth)
                    .map_err(MethodError::Unauthorised)?;
                // Update password requires token key type.
                let user =
                    pattern::user_read_email_checked(driver, Some(&service), audit, &req.email)
                        .map_err(MethodError::BadRequest)?;
                let key =
                    pattern::key_read_user_checked(driver, &service, audit, &user, KeyType::Token)
                        .map_err(MethodError::BadRequest)?;
                // User is allowed to update password if `password_require_update` is true.
                // Check user password.
                user.password_check(&req.password)
                    .map_err(MethodError::BadRequest)?;
                // Encode revoke token.
                let token =
                    Jwt::encode_revoke_token(driver, &service, &user, &key, revoke_token_expires)
                        .map_err(MethodError::BadRequest)?;
                // Update user password.
                let user_update = UserUpdate::new_password(user.id, &req.new_password)
                    .map_err(MethodError::BadRequest)?;
                driver
                    .user_update(&user_update)
                    .map_err(MethodError::BadRequest)?;
                let user =
                    pattern::user_read_email_checked(driver, Some(&service), audit, &req.email)
                        .map_err(MethodError::BadRequest)?;
                // Send update password email.
                TemplateEmail::email_update_password(&service, &user, &token, audit.meta())
                    .map_err(MethodError::BadRequest)
            },
        )?;
        email(template)
            .map_err::<DriverError, _>(Into::into)
            .map_err(MethodError::BadRequest)?;
        Ok(password_meta)
    })
    .await
    .map(|password_meta| pb::AuthPasswordMetaReply {
        meta: Some(password_meta.into()),
    })
}

pub async fn update_password_revoke(
    server: &Server,
    request: MethodRequest<pb::AuthTokenRequest>,
) -> MethodResult<pb::AuthAuditReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    method_blocking(move || {
        audit_result(
            driver.as_ref(),
            audit_meta,
            AuditType::AuthLocalUpdatePasswordRevoke,
            |driver, audit| revoke_inner(driver, audit, &auth, &req),
        )
        .map_err(Into::into)
    })
    .await
    .map(|audit| pb::AuthAuditReply {
        audit: uuid_opt_to_string_opt(audit.map(|x| x.id)),
    })
}

fn revoke_inner(
    driver: &Postgres,
    audit: &mut AuditBuilder,
    auth: &HeaderAuth,
    req: &pb::AuthTokenRequest,
) -> MethodResult<Option<Audit>> {
    let service = pattern::key_service_authenticate(driver, audit, auth)
        .map_err(MethodError::Unauthorised)?;
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) =
        Jwt::decode_unsafe(&req.token, service.id).map_err(MethodError::BadRequest)?;
    // Update email revoke requires token key type.
    // Do not check user, key is enabled or not revoked.
    let user = pattern::user_read_id_unchecked(driver, Some(&service), audit, user_id)
        .map_err(MethodError::BadRequest)?;
    let key = pattern::key_read_user_unchecked(driver, &service, audit, &user, KeyType::Token)
        .map_err(MethodError::BadRequest)?;
    // Safely decode token with user key.
    let csrf_key = Jwt::decode_revoke_token(&service, &user, &key, &req.token)
        .map_err(MethodError::BadRequest)?;
    // Verify CSRF to prevent reuse.
    api_csrf_verify(driver, &service, &csrf_key)?;

    // Disable user and disable and revoke all keys associated with user.
    driver
        .user_update(&UserUpdate::new_id(user.id).set_is_enabled(false))
        .map_err(MethodError::BadRequest)?;
    driver
        .key_update_many(
            &user.id,
            &KeyUpdate {
                id: Uuid::nil(),
                is_enabled: Some(false),
                is_revoked: Some(true),
                name: None,
            },
        )
        .map_err(MethodError::BadRequest)?;
    // Optionally create custom audit log.
    if let Some(x) = &req.audit {
        let audit = audit
            .create(driver, x, None, None)
            .map_err(MethodError::BadRequest)?;
        Ok(Some(audit))
    } else {
        Ok(None)
    }
}
