use crate::{
    notify_msg::{EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword},
    AuditBuilder, AuditData, AuditMessage, AuditPath, CoreError, CoreResult, Csrf, Driver, Jwt,
    JwtClaimsType, Key, NotifyActor, Service, User, UserAccessToken, UserKey, UserToken,
};
use actix::Addr;
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
            Auth::user_read_by_email(driver, Some(service), audit, AuditPath::LoginError, email)?;
        let key = Auth::key_read_by_user(driver, service, audit, AuditPath::LoginError, &user)?;

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            audit.create_internal(
                driver,
                AuditPath::LoginError,
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

        audit.create_internal(driver, AuditPath::Login, AuditMessage::Login);
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
            AuditPath::ResetPasswordError,
            email,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditPath::ResetPasswordError, &user)?;

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
            AuditPath::ResetPassword,
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
            AuditPath::ResetPasswordConfirmError,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            driver,
            service,
            audit,
            AuditPath::ResetPasswordConfirmError,
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
                    AuditPath::ResetPasswordConfirmError,
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
            AuditPath::ResetPasswordConfirmError,
        )?;

        // Sucessful reset password confirm, update user password.
        let count = User::update_password_by_id(driver, Some(service), audit, user.id, password)?;

        audit.create_internal(
            driver,
            AuditPath::ResetPasswordConfirm,
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
            AuditPath::UpdateEmailError,
            user_id,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditPath::UpdateEmailError, &user)?;
        let old_email = user.email.to_owned();

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            audit.create_internal(
                driver,
                AuditPath::UpdateEmailError,
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
            AuditPath::UpdateEmailError,
            user_id,
        )?;

        // Pass audit log to notification actor.
        let audit =
            audit.create_internal(driver, AuditPath::UpdateEmail, AuditMessage::UpdateEmail);

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
            AuditPath::UpdateEmailRevokeError,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            driver,
            service,
            audit,
            AuditPath::UpdateEmailRevokeError,
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
                    AuditPath::UpdateEmailRevokeError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_check(driver, &csrf_key, &audit, AuditPath::UpdateEmailRevokeError)?;

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
            AuditPath::UpdateEmailRevoke,
            AuditMessage::UpdateEmailRevoke,
        );
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
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
            AuditPath::UpdatePasswordError,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            driver,
            service,
            audit,
            AuditPath::UpdatePasswordError,
            &user,
        )?;

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            audit.create_internal(
                driver,
                AuditPath::UpdatePasswordError,
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
            AuditPath::UpdatePasswordError,
            user_id,
        )?;

        // Pass audit log to notification actor.
        let audit = audit.create_internal(
            driver,
            AuditPath::UpdatePassword,
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
            AuditPath::UpdatePasswordRevokeError,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            driver,
            service,
            audit,
            AuditPath::UpdatePasswordRevokeError,
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
                    AuditPath::UpdatePasswordRevokeError,
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
            AuditPath::UpdatePasswordRevokeError,
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
            AuditPath::UpdatePasswordRevoke,
            AuditMessage::UpdatePasswordRevoke,
        );
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
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
            Auth::key_read_by_user_value(driver, service, audit, AuditPath::KeyVerifyError, key)?;

        // Check key is associated with user.
        let user_id = match key.user_id.ok_or_else(|| CoreError::BadRequest) {
            Ok(user_id) => user_id,
            Err(err) => {
                audit.create_internal(driver, AuditPath::KeyVerifyError, AuditMessage::KeyNotFound);
                return Err(err);
            }
        };

        // Successful key verify.
        let user_key = UserKey {
            user_id,
            key: key.value,
        };

        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
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
            AuditPath::KeyRevokeError,
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

        audit.create_internal(driver, AuditPath::KeyRevoke, AuditMessage::KeyRevoke);
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
        }
        Ok(1)
    }

    pub fn token_verify(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        token: &str,
        audit_data: Option<&AuditData>,
    ) -> CoreResult<UserAccessToken> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(token, service.id)?;

        let user = Auth::user_read_by_id(
            driver,
            Some(service),
            audit,
            AuditPath::TokenVerifyError,
            user_id,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditPath::TokenVerifyError, &user)?;

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
                    AuditPath::TokenVerifyError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Successful token verify.
        let user_token = UserAccessToken {
            user_id: user.id.to_owned(),
            access_token: token.to_owned(),
            access_token_expires,
        };

        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
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
            AuditPath::TokenRefreshError,
            user_id,
        )?;
        let key =
            Auth::key_read_by_user(driver, service, audit, AuditPath::TokenRefreshError, &user)?;

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
                    AuditPath::TokenRefreshError,
                    AuditMessage::TokenInvalidOrExpired,
                );
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_check(driver, &csrf_key, &audit, AuditPath::TokenRefreshError)?;

        // Successful token refresh, encode user token.
        let user_token = Auth::encode_user_token(
            driver,
            &service,
            &user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        audit.create_internal(driver, AuditPath::TokenRefresh, AuditMessage::TokenRefresh);
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
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
            AuditPath::TokenRevokeError,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            driver,
            service,
            audit,
            AuditPath::TokenRevokeError,
            &user,
        )?;

        // Safely decode token with user key.
        let csrf_key = match Jwt::decode_token(service.id, user.id, token_type, &key.value, token) {
            Ok((_, csrf_key)) => csrf_key,
            Err(err) => {
                audit.create_internal(
                    driver,
                    AuditPath::TokenRevokeError,
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

        audit.create_internal(driver, AuditPath::TokenRevoke, AuditMessage::TokenRevoke);
        if let Some(audit_data) = audit_data {
            audit.create_unchecked(driver, &audit_data.path, &audit_data.data);
        }
        Ok(1)
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
            Auth::service_read_by_id(driver, service_id, audit, AuditPath::Oauth2LoginError)?;
        let user = Auth::user_read_by_email(
            driver,
            Some(&service),
            audit,
            AuditPath::Oauth2LoginError,
            email,
        )?;
        let key =
            Auth::key_read_by_user(driver, &service, audit, AuditPath::Oauth2LoginError, &user)?;

        // Successful OAuth2 login, return service for redirect callback integration.
        let user_token = Auth::encode_user_token(
            driver,
            &service,
            &user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        audit.create_internal(driver, AuditPath::Oauth2Login, AuditMessage::Oauth2Login);
        Ok((service, user_token))
    }

    /// Read service by ID.
    /// Also checks service is enabled, returns bad request if disabled.
    fn service_read_by_id(
        driver: &dyn Driver,
        service_id: Uuid,
        audit: &mut AuditBuilder,
        audit_path: AuditPath,
    ) -> CoreResult<Service> {
        match driver
            .service_read_by_id(service_id)
            .map_err(CoreError::Driver)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(service) => {
                audit.set_service(Some(&service));
                if !service.is_enabled {
                    audit.create_internal(driver, audit_path, AuditMessage::ServiceDisabled);
                    return Err(CoreError::BadRequest);
                }
                Ok(service)
            }
            Err(err) => {
                audit.create_internal(driver, audit_path, AuditMessage::ServiceNotFound);
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
        audit_path: AuditPath,
        id: Uuid,
    ) -> CoreResult<User> {
        match User::read_by_id(driver, service_mask, audit, id)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(user) => {
                audit.set_user(Some(&user));
                if !user.is_enabled {
                    audit.create_internal(driver, audit_path, AuditMessage::UserDisabled);
                    return Err(CoreError::BadRequest);
                }
                Ok(user)
            }
            Err(err) => {
                audit.create_internal(driver, audit_path, AuditMessage::UserNotFound);
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
        audit_path: AuditPath,
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
                audit.create_internal(driver, audit_path, AuditMessage::UserNotFound);
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
        audit_path: AuditPath,
        email: &str,
    ) -> CoreResult<User> {
        match User::read_by_email(driver, service_mask, audit, email)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(user) => {
                audit.set_user(Some(&user));
                if !user.is_enabled {
                    audit.create_internal(driver, audit_path, AuditMessage::UserDisabled);
                    return Err(CoreError::BadRequest);
                }
                Ok(user)
            }
            Err(err) => {
                audit.create_internal(driver, audit_path, AuditMessage::UserNotFound);
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
        audit_path: AuditPath,
        user: &User,
    ) -> CoreResult<Key> {
        match Key::read_by_user(driver, &service, audit, &user)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_internal(driver, audit_path, AuditMessage::KeyDisabledOrRevoked);
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_path, AuditMessage::KeyNotFound);
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
        audit_path: AuditPath,
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
                audit.create_internal(driver, audit_path, AuditMessage::KeyNotFound);
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
        audit_path: AuditPath,
        key: &str,
    ) -> CoreResult<Key> {
        match Key::read_by_user_value(driver, service, audit, key)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.set_user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_internal(driver, audit_path, AuditMessage::KeyDisabledOrRevoked);
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_internal(driver, audit_path, AuditMessage::KeyNotFound);
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
        audit_path: AuditPath,
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
                audit.create_internal(driver, audit_path, AuditMessage::KeyNotFound);
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
        audit_path: AuditPath,
    ) -> CoreResult<()> {
        let res = Csrf::read_by_key(driver, csrf_key)?
            .ok_or_else(|| CoreError::BadRequest)
            .map(|_| ());

        match res {
            Ok(_) => Ok(()),
            Err(err) => {
                audit.create_internal(driver, audit_path, AuditMessage::CsrfNotFoundOrUsed);
                Err(err)
            }
        }
    }
}
