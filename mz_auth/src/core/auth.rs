use crate::{
    notify_msg::{EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword},
    AuditBuilder, AuditData, AuditMessage, AuditType, CoreError, CoreResult, Csrf, Driver, Jwt,
    JwtClaimsType, Key, NotifyActor, Service, User, UserKey, UserToken, UserTokenAccess,
};
use actix::Addr;
use libreauth::oath::TOTPBuilder;
use uuid::Uuid;

/// Authentication.
pub struct Auth;

impl Auth {
    pub fn login(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        email: &str,
        password: &str,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<UserToken> {
        let user =
            Auth::user_read_by_email(driver, Some(service), audit, AuditType::LoginError, email)?;
        let key = Auth::key_read_by_user(driver, service, audit, AuditType::LoginError, &user)?;

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            audit.create_internal(
                driver,
                AuditType::LoginError,
                AuditMessage::PasswordNotSetOrIncorrect,
            );
            return Err(err);
        }

        // Successful login, encode and return user token.
        let user_token = Auth::encode_user_token(
            driver,
            &service,
            &user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        audit.create_internal(driver, AuditType::Login, AuditMessage::Login);
        Ok(user_token)
    }

    /// Reset user password via email request. In case of an error this function
    /// returns Ok to prevent the caller from inferring the existence of a user.
    pub fn reset_password(
        driver: &dyn Driver,
        notify: &Addr<NotifyActor>,
        service: &Service,
        audit: &mut AuditBuilder,
        email: &str,
        token_expires: i64,
    ) -> CoreResult<()> {
        Auth::reset_password_inner(driver, notify, service, audit, email, token_expires)
            .or_else(|_err| Ok(()))
    }

    fn reset_password_inner(
        driver: &dyn Driver,
        notify: &Addr<NotifyActor>,
        service: &Service,
        audit: &mut AuditBuilder,
        email: &str,
        token_expires: i64,
    ) -> CoreResult<()> {
        let user = Auth::user_read_by_email(
            driver,
            Some(service),
            audit,
            AuditType::ResetPasswordError,
            email,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditType::ResetPasswordError, &user)?;

        // Successful reset password, encode reset token.
        let csrf = Auth::csrf_create(driver, service, token_expires)?;
        let (token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;

        // Pass audit log to notification actor.
        let audit = audit.create_internal(
            driver,
            AuditType::ResetPassword,
            AuditMessage::ResetPassword,
        );

        // Send reset password action email.
        notify
            .try_send(EmailResetPassword::new(service.clone(), user, token, audit))
            .map_err(|_err| CoreError::BadRequest)?;
        Ok(())
    }

    pub fn reset_password_confirm(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        password: &str,
    ) -> CoreResult<usize> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(token, service.id)?;

        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::ResetPasswordConfirmError,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            driver,
            service,
            audit,
            AuditType::ResetPasswordConfirmError,
            &user,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &key.value,
            token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditType::ResetPasswordConfirmError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_check(
            driver,
            &csrf_key,
            &audit,
            AuditType::ResetPasswordConfirmError,
        )?;

        // Sucessful reset password confirm, update user password.
        let count = User::update_password_by_id(driver, Some(service), audit, user.id, password)?;

        audit.create_internal(
            driver,
            AuditType::ResetPasswordConfirm,
            AuditMessage::ResetPasswordConfirm,
        );
        Ok(count)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_email(
        driver: &dyn Driver,
        notify: &Addr<NotifyActor>,
        service: &Service,
        audit: &mut AuditBuilder,
        key: Option<&str>,
        token: Option<&str>,
        password: &str,
        new_email: &str,
        revoke_token_expires: i64,
    ) -> CoreResult<()> {
        // Verify key or token argument to get user ID.
        let user_id = Auth::key_or_token_verify(driver, service, audit, key, token)?;

        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::UpdateEmailError,
            user_id,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditType::UpdateEmailError, &user)?;
        let old_email = user.email.to_owned();

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            audit.create_internal(
                driver,
                AuditType::UpdateEmailError,
                AuditMessage::PasswordNotSetOrIncorrect,
            );
            return Err(err);
        }

        // Successful update email, encode revoke token.
        let csrf = Auth::csrf_create(driver, service, revoke_token_expires)?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &csrf.key,
            &key.value,
            revoke_token_expires,
        )?;

        // Update user email.
        User::update_email_by_id(driver, Some(service), audit, user.id, new_email)?;
        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::UpdateEmailError,
            user_id,
        )?;

        // Pass audit log to notification actor.
        let audit =
            audit.create_internal(driver, AuditType::UpdateEmail, AuditMessage::UpdateEmail);

        // Send update email action email.
        notify
            .try_send(EmailUpdateEmail::new(
                service.clone(),
                user,
                old_email,
                revoke_token,
                audit,
            ))
            .map_err(|_err| CoreError::BadRequest)?;
        Ok(())
    }

    pub fn update_email_revoke(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<usize> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(token, service.id)?;

        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(
            driver,
            Some(service),
            audit,
            AuditType::UpdateEmailRevokeError,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            driver,
            service,
            audit,
            AuditType::UpdateEmailRevokeError,
            &user,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &key.value,
            token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditType::UpdateEmailRevokeError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_check(driver, &csrf_key, &audit, AuditType::UpdateEmailRevokeError)?;

        // Successful update email revoke, disable user and disable and revoke all keys associated with user.
        User::update_by_id(driver, Some(service), audit, user.id, Some(false), None)?;
        let count = Key::update_many_by_user_id(
            driver,
            Some(service),
            audit,
            user.id,
            Some(false),
            Some(true),
            None,
        )?;

        audit.create_internal(
            driver,
            AuditType::UpdateEmailRevoke,
            AuditMessage::UpdateEmailRevoke,
        );
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(count + 1)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_password(
        driver: &dyn Driver,
        notify: &Addr<NotifyActor>,
        service: &Service,
        audit: &mut AuditBuilder,
        key: Option<&str>,
        token: Option<&str>,
        password: &str,
        new_password: &str,
        revoke_token_expires: i64,
    ) -> CoreResult<()> {
        // Verify key or token argument to get user ID.
        let user_id = Auth::key_or_token_verify(driver, service, audit, key, token)?;

        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::UpdatePasswordError,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            driver,
            service,
            audit,
            AuditType::UpdatePasswordError,
            &user,
        )?;

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            audit.create_internal(
                driver,
                AuditType::UpdatePasswordError,
                AuditMessage::PasswordNotSetOrIncorrect,
            );
            return Err(err);
        }

        // Successful update password, encode revoke token.
        let csrf = Auth::csrf_create(driver, service, revoke_token_expires)?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
            &csrf.key,
            &key.value,
            revoke_token_expires,
        )?;

        // Update user password, reread from driver.
        User::update_password_by_id(driver, Some(service), audit, user.id, new_password)?;
        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::UpdatePasswordError,
            user_id,
        )?;

        // Pass audit log to notification actor.
        let audit = audit.create_internal(
            driver,
            AuditType::UpdatePassword,
            AuditMessage::UpdatePassword,
        );

        // Send update password action email.
        notify
            .try_send(EmailUpdatePassword::new(
                service.clone(),
                user,
                revoke_token,
                audit,
            ))
            .map_err(|_err| CoreError::BadRequest)?;
        Ok(())
    }

    pub fn update_password_revoke(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<usize> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(token, service.id)?;

        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(
            driver,
            Some(service),
            audit,
            AuditType::UpdatePasswordRevokeError,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            driver,
            service,
            audit,
            AuditType::UpdatePasswordRevokeError,
            &user,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
            &key.value,
            token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditType::UpdatePasswordRevokeError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_check(
            driver,
            &csrf_key,
            &audit,
            AuditType::UpdatePasswordRevokeError,
        )?;

        // Successful update password revoke, disable user and disable and revoke all keys associated with user.
        User::update_by_id(driver, Some(service), audit, user.id, Some(false), None)?;
        let count = Key::update_many_by_user_id(
            driver,
            Some(service),
            audit,
            user.id,
            Some(false),
            Some(true),
            None,
        )?;

        audit.create_internal(
            driver,
            AuditType::UpdatePasswordRevoke,
            AuditMessage::UpdatePasswordRevoke,
        );
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(count + 1)
    }

    pub fn key_verify(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        key: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<UserKey> {
        let key =
            Auth::key_read_by_user_value(driver, service, audit, AuditType::KeyVerifyError, key)?;

        // Check key is associated with user.
        let user_id = match key.user_id.ok_or_else(|| CoreError::BadRequest) {
            Ok(user_id) => user_id,
            Err(err) => {
                audit.create_internal(driver, AuditType::KeyVerifyError, AuditMessage::KeyNotFound);
                return Err(err);
            }
        };

        // Successful key verify.
        let user_key = UserKey {
            user_id,
            key: key.value,
        };

        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(user_key)
    }

    pub fn key_revoke(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        key: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<usize> {
        // Do not check key is enabled or not revoked.
        let key = Auth::key_read_by_user_value_unchecked(
            driver,
            service,
            audit,
            AuditType::KeyRevokeError,
            key,
        )?;

        // Successful key revoke, disable and revoke key.
        Key::update_by_id(
            driver,
            Some(service),
            audit,
            key.id,
            Some(false),
            Some(true),
            None,
        )?;

        audit.create_internal(driver, AuditType::KeyRevoke, AuditMessage::KeyRevoke);
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(1)
    }

    pub fn token_verify(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<UserTokenAccess> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(token, service.id)?;

        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::TokenVerifyError,
            user_id,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditType::TokenVerifyError, &user)?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::AccessToken,
            &key.value,
            token,
        );
        let access_token_expires = match decoded {
            Ok((access_token_expires, _)) => access_token_expires,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditType::TokenVerifyError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Successful token verify.
        let user_token = UserTokenAccess {
            user_id: user.id.to_owned(),
            access_token: token.to_owned(),
            access_token_expires,
        };

        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(user_token)
    }

    pub fn token_refresh(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        audit_data: Option<&AuditData>,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<UserToken> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(token, service.id)?;

        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditType::TokenRefreshError,
            user_id,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditType::TokenRefreshError, &user)?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            service.id,
            user.id,
            JwtClaimsType::RefreshToken,
            &key.value,
            token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditType::TokenRefreshError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_check(driver, &csrf_key, &audit, AuditType::TokenRefreshError)?;

        // Successful token refresh, encode user token.
        let user_token = Auth::encode_user_token(
            driver,
            &service,
            &user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        audit.create_internal(driver, AuditType::TokenRefresh, AuditMessage::TokenRefresh);
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(user_token)
    }

    pub fn token_revoke(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<usize> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, token_type) = Jwt::decode_unsafe(token, service.id)?;

        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(
            driver,
            Some(service),
            audit,
            AuditType::TokenRevokeError,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            driver,
            service,
            audit,
            AuditType::TokenRevokeError,
            &user,
        )?;

        // Safely decode token with user key.
        let csrf_key = match Jwt::decode_token(service.id, user.id, token_type, &key.value, token) {
            Ok((_, csrf_key)) => csrf_key,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditType::TokenRevokeError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // If token has CSRF key, invalidate it now.
        if let Some(csrf_key) = csrf_key {
            Csrf::read_by_key(driver, &csrf_key)?;
        }

        // Successful token revoke, disable and revoke key associated with token.
        Key::update_by_id(
            driver,
            Some(service),
            audit,
            key.id,
            Some(false),
            Some(true),
            None,
        )?;

        audit.create_internal(driver, AuditType::TokenRevoke, AuditMessage::TokenRevoke);
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.type_, &audit_data.data);
        }
        Ok(1)
    }

    /// TOTP code verification.
    pub fn totp(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        key_id: Uuid,
        totp_code: &str,
    ) -> CoreResult<()> {
        // TODO(!docs): Add guide, documentation for TOTP.
        // TODO(!test): Add tests for TOTP.
        let key = Auth::key_read_by_id(driver, service, audit, AuditType::TotpError, key_id)?;
        let totp = TOTPBuilder::new()
            .hex_key(&key.value)
            .finalize()
            .map_err(CoreError::libreauth_oath)?;

        if !totp.is_valid(totp_code) {
            audit.create_internal(driver, AuditType::TotpError, AuditMessage::TotpInvalid);
            Err(CoreError::BadRequest)
        } else {
            Ok(())
        }
    }

    /// OAuth2 user login.
    pub fn oauth2_login(
        driver: &dyn Driver,
        service_id: Uuid,
        audit: &mut AuditBuilder,
        email: &str,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<(Service, UserToken)> {
        let service =
            Auth::service_read_by_id(driver, service_id, audit, AuditType::Oauth2LoginError)?;
        let user = Auth::user_read_by_email(
            driver,
            Some(&service),
            audit,
            AuditType::Oauth2LoginError,
            email,
        )?;
        let key =
            Auth::key_read_by_user(driver, &service, audit, AuditType::Oauth2LoginError, &user)?;

        // Successful OAuth2 login, return service for redirect callback integration.
        let user_token = Auth::encode_user_token(
            driver,
            &service,
            &user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        audit.create_internal(driver, AuditType::Oauth2Login, AuditMessage::Oauth2Login);
        Ok((service, user_token))
    }

    /// Read service by ID.
    /// Also checks service is enabled, returns bad request if disabled.
    fn service_read_by_id(
        driver: &dyn Driver,
        service_id: Uuid,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
    ) -> CoreResult<Service> {
        match driver
            .service_read_opt(&service_id)
            .map_err(CoreError::Driver)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(service) => {
                audit.set_service(Some(&service));
                if !service.is_enabled {
                    audit.create_internal(driver, audit_type, AuditMessage::ServiceDisabled);
                    return Err(CoreError::BadRequest);
                }
                Ok(service)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::ServiceNotFound);
                Err(err)
            }
        }
    }

    /// Read user by ID.
    /// Checks user is enabled, returns bad request if disabled.
    fn user_read_by_id(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        id: Uuid,
    ) -> CoreResult<User> {
        match User::read_by_id(driver, service_mask, audit, id)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(user) => {
                audit.set_user(Some(&user));
                if !user.is_enabled {
                    audit.create_internal(driver, audit_type, AuditMessage::UserDisabled);
                    return Err(CoreError::BadRequest);
                }
                Ok(user)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::UserNotFound);
                Err(err)
            }
        }
    }

    /// Unchecked read user by ID.
    /// Does not check user is enabled.
    fn user_read_by_id_unchecked(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        id: Uuid,
    ) -> CoreResult<User> {
        match User::read_by_id(driver, service_mask, audit, id)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(user) => {
                audit.set_user(Some(&user));
                Ok(user)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::UserNotFound);
                Err(err)
            }
        }
    }

    /// Read user by email address.
    /// Also checks user is enabled, returns bad request if disabled.
    fn user_read_by_email(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        email: &str,
    ) -> CoreResult<User> {
        match User::read_by_email(driver, service_mask, audit, email)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(user) => {
                audit.set_user(Some(&user));
                if !user.is_enabled {
                    audit.create_internal(driver, audit_type, AuditMessage::UserDisabled);
                    return Err(CoreError::BadRequest);
                }
                Ok(user)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::UserNotFound);
                Err(err)
            }
        }
    }

    /// Read key by ID.
    /// Also checks key is enabled and not revoked, returns bad request if disabled.
    fn key_read_by_id(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        key_id: Uuid,
    ) -> CoreResult<Key> {
        match Key::read_by_id(driver, Some(&service), audit, key_id)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_internal(driver, audit_type, AuditMessage::KeyDisabledOrRevoked);
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::KeyNotFound);
                Err(err)
            }
        }
    }

    /// Read key by user reference.
    /// Also checks key is enabled and not revoked, returns bad request if disabled.
    fn key_read_by_user(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        user: &User,
    ) -> CoreResult<Key> {
        match Key::read_by_user(driver, &service, audit, &user)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_internal(driver, audit_type, AuditMessage::KeyDisabledOrRevoked);
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::KeyNotFound);
                Err(err)
            }
        }
    }

    /// Unchecked read key by user reference.
    /// Does not check key is enabled or not revoked.
    fn key_read_by_user_unchecked(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        user: &User,
    ) -> CoreResult<Key> {
        match Key::read_by_user(driver, &service, audit, &user)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::KeyNotFound);
                Err(err)
            }
        }
    }

    /// Read key by user value.
    /// Also checks key is enabled and not revoked, returns bad request if disabled.
    fn key_read_by_user_value(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        key: &str,
    ) -> CoreResult<Key> {
        match Key::read_by_user_value(driver, service, audit, key)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_internal(driver, audit_type, AuditMessage::KeyDisabledOrRevoked);
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::KeyNotFound);
                Err(err)
            }
        }
    }

    /// Unchecked read key by user value.
    /// Does not check key is enabled and not revoked.
    fn key_read_by_user_value_unchecked(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        key: &str,
    ) -> CoreResult<Key> {
        match Key::read_by_user_value(driver, service, audit, key)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::KeyNotFound);
                Err(err)
            }
        }
    }

    /// Get user ID from valid key or token.
    fn key_or_token_verify(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        key: Option<&str>,
        token: Option<&str>,
    ) -> CoreResult<Uuid> {
        match key {
            Some(key) => {
                let user_key = Auth::key_verify(driver, service, audit, key, None)?;
                Ok(user_key.user_id)
            }
            None => match token {
                Some(token) => {
                    let user_token = Auth::token_verify(driver, service, audit, token, None)?;
                    Ok(user_token.user_id)
                }
                None => Err(CoreError::Forbidden),
            },
        }
    }

    /// Build user token by encoding access and refresh tokens.
    fn encode_user_token(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        key: &Key,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<UserToken> {
        let csrf = Auth::csrf_create(driver, &service, refresh_token_expires)?;
        let (access_token, access_token_expires) = Jwt::encode_token(
            service.id,
            user.id,
            JwtClaimsType::AccessToken,
            &key.value,
            access_token_expires,
        )?;
        let (refresh_token, refresh_token_expires) = Jwt::encode_token_csrf(
            service.id,
            user.id,
            JwtClaimsType::RefreshToken,
            &csrf.key,
            &key.value,
            refresh_token_expires,
        )?;
        Ok(UserToken {
            user_id: user.id.to_owned(),
            access_token,
            access_token_expires,
            refresh_token,
            refresh_token_expires,
        })
    }

    /// Create a new CSRF key, value pair using random UUID.
    fn csrf_create(driver: &dyn Driver, service: &Service, token_expires: i64) -> CoreResult<Csrf> {
        let csrf_key = Key::value_generate();
        Csrf::create(driver, service, &csrf_key, &csrf_key, token_expires)
    }

    /// Check a CSRF key is valid by reading it, this will also delete the key.
    fn csrf_check(
        driver: &dyn Driver,
        csrf_key: &str,
        audit: &AuditBuilder,
        audit_type: AuditType,
    ) -> CoreResult<()> {
        let res = Csrf::read_by_key(driver, csrf_key)?
            .ok_or_else(|| CoreError::BadRequest)
            .map(|_| ());

        match res {
            Ok(_) => Ok(()),
            Err(err) => {
                audit.create_internal(driver, audit_type, AuditMessage::CsrfNotFoundOrUsed);
                Err(err)
            }
        }
    }
}
