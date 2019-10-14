use crate::{
    notify_msg::{EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword},
    Audit, AuditBuilder, AuditCreate2, AuditMessage, AuditType, Core, CoreError, CoreResult, Csrf,
    Driver, Jwt, JwtClaimsType, Key, KeyType, KeyWithValue, NotifyActor, Service, User, UserKey,
    UserRead, UserToken, UserTokenAccess, UserUpdate,
};
use actix::Addr;
use libreauth::oath::TOTPBuilder;
use std::fmt;
use uuid::Uuid;

/// Authentication functions common arguments.
pub struct AuthArgs<'a> {
    driver: &'a dyn Driver,
    service: &'a Service,
    audit: &'a mut AuditBuilder,
}

impl<'a> AuthArgs<'a> {
    pub fn new(driver: &'a dyn Driver, service: &'a Service, audit: &'a mut AuditBuilder) -> Self {
        Self {
            driver,
            service,
            audit,
        }
    }
}

impl<'a> fmt::Debug for AuthArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AuthArgs {{ driver, service: {:?}, audit: {:?} }}",
            self.service, self.audit
        )
    }
}

/// Authentication functions.
#[derive(Debug)]
pub struct Auth;

impl Auth {
    pub fn login(
        args: AuthArgs,
        email: String,
        password: String,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<UserToken> {
        // Login requires token key type.
        let user = Auth::user_read_by_email(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalLogin,
            email,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalLogin,
            &user,
            KeyType::Token,
        )?;

        // If user password update required, return forbidden.
        if user.password_require_update {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalLogin,
                AuditMessage::PasswordUpdateRequired,
            )?;
            return Err(CoreError::Forbidden);
        }

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalLogin,
                AuditMessage::PasswordNotSetOrIncorrect,
            )?;
            return Err(err);
        }

        // Successful login, encode and return user token.
        let user_token = Auth::encode_user_token(
            args.driver,
            args.service,
            args.audit,
            user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        args.audit
            .create_message(args.driver, AuditType::AuthLocalLogin, AuditMessage::Login)?;
        Ok(user_token)
    }

    /// Reset user password via email request. In case of an error this function
    /// returns Ok to prevent the caller from inferring the existence of a user.
    pub fn reset_password(
        args: AuthArgs,
        notify: &Addr<NotifyActor>,
        email: String,
        token_expires: i64,
    ) -> CoreResult<()> {
        Auth::reset_password_inner(args, notify, email, token_expires).or_else(|_err| Ok(()))
    }

    fn reset_password_inner(
        args: AuthArgs,
        notify: &Addr<NotifyActor>,
        email: String,
        token_expires: i64,
    ) -> CoreResult<()> {
        // Reset password requires token key type.
        let user = Auth::user_read_by_email(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalResetPassword,
            email,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalResetPassword,
            &user,
            KeyType::Token,
        )?;

        // Check user password reset is allowed.
        if !user.password_allow_reset {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalResetPassword,
                AuditMessage::ResetPasswordDisabled,
            )?;
            return Err(CoreError::BadRequest);
        }

        // Successful reset password, encode reset token.
        let csrf = Auth::csrf_create_inner(args.driver, args.service, token_expires)?;
        let (token, _) = Jwt::encode_token_csrf(
            args.service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &csrf.key,
            &key.value,
            token_expires,
        )?;

        // Pass audit log to notification actor.
        let audit = args.audit.create_message(
            args.driver,
            AuditType::AuthLocalResetPassword,
            AuditMessage::ResetPassword,
        )?;

        // Send reset password action email.
        notify
            .try_send(EmailResetPassword::new(
                args.service.clone(),
                user,
                token,
                Some(audit),
            ))
            .map_err(|_err| CoreError::BadRequest)?;
        Ok(())
    }

    pub fn reset_password_confirm(
        args: AuthArgs,
        token: String,
        password: String,
    ) -> CoreResult<()> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(&token, args.service.id)?;

        // Reset password confirm requires token key type.
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalResetPasswordConfirm,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalResetPasswordConfirm,
            &user,
            KeyType::Token,
        )?;

        // Check user password reset is allowed.
        if !user.password_allow_reset {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalResetPasswordConfirm,
                AuditMessage::ResetPasswordDisabled,
            )?;
            return Err(CoreError::BadRequest);
        }

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            args.service.id,
            user.id,
            JwtClaimsType::ResetPasswordToken,
            &key.value,
            &token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                args.audit.create_message(
                    args.driver,
                    AuditType::AuthLocalResetPasswordConfirm,
                    AuditMessage::TokenInvalidOrExpired,
                )?;
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_verify_inner(
            args.driver,
            args.service,
            args.audit,
            csrf_key,
            AuditType::AuthLocalResetPasswordConfirm,
        )?;

        // Sucessful reset password confirm, update user password.
        User::update_password(args.driver, Some(args.service), user.id, password)?;
        args.audit.create_message(
            args.driver,
            AuditType::AuthLocalResetPasswordConfirm,
            AuditMessage::ResetPasswordConfirm,
        )?;
        Ok(())
    }

    pub fn update_email(
        args: AuthArgs,
        notify: &Addr<NotifyActor>,
        user_id: Uuid,
        password: String,
        new_email: String,
        revoke_token_expires: i64,
    ) -> CoreResult<()> {
        // Update email requires token key type.
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalUpdateEmail,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalUpdateEmail,
            &user,
            KeyType::Token,
        )?;
        let old_email = user.email.to_owned();

        // If user password update required, return forbidden.
        if user.password_require_update {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalUpdateEmail,
                AuditMessage::PasswordUpdateRequired,
            )?;
            return Err(CoreError::Forbidden);
        }

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalUpdateEmail,
                AuditMessage::PasswordNotSetOrIncorrect,
            )?;
            return Err(err);
        }

        // Successful update email, encode revoke token.
        let csrf = Auth::csrf_create_inner(args.driver, args.service, revoke_token_expires)?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            args.service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &csrf.key,
            &key.value,
            revoke_token_expires,
        )?;

        // Update user email.
        User::update_email(args.driver, Some(args.service), user.id, new_email)?;
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalUpdateEmail,
            user_id,
        )?;

        // Pass audit log to notification actor.
        let audit = args.audit.create_message(
            args.driver,
            AuditType::AuthLocalUpdateEmail,
            AuditMessage::UpdateEmail,
        )?;

        // Send update email action email.
        notify
            .try_send(EmailUpdateEmail::new(
                args.service.clone(),
                user,
                old_email,
                revoke_token,
                Some(audit),
            ))
            .map_err(|_err| CoreError::BadRequest)?;
        Ok(())
    }

    pub fn update_email_revoke(
        args: AuthArgs,
        token: String,
        audit_create: Option<AuditCreate2>,
    ) -> CoreResult<Option<Audit>> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(&token, args.service.id)?;

        // Update email revoke requires token key type.
        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalUpdateEmailRevoke,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalUpdateEmailRevoke,
            &user,
            KeyType::Token,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            args.service.id,
            user.id,
            JwtClaimsType::UpdateEmailRevokeToken,
            &key.value,
            &token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                args.audit.create_message(
                    args.driver,
                    AuditType::AuthLocalUpdateEmailRevoke,
                    AuditMessage::TokenInvalidOrExpired,
                )?;
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_verify_inner(
            args.driver,
            args.service,
            args.audit,
            csrf_key,
            AuditType::AuthLocalUpdateEmailRevoke,
        )?;

        // Successful update email revoke, disable user and disable and revoke all keys associated with user.
        let update = UserUpdate {
            is_enabled: Some(false),
            name: None,
            locale: None,
            timezone: None,
            password_allow_reset: None,
            password_require_update: None,
        };
        User::update(args.driver, Some(args.service), user.id, &update)?;
        Key::update_many(
            args.driver,
            Some(args.service),
            user.id,
            Some(false),
            Some(true),
            None,
        )?;

        args.audit.create_message(
            args.driver,
            AuditType::AuthLocalUpdateEmailRevoke,
            AuditMessage::UpdateEmailRevoke,
        )?;
        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok(audit)
    }

    pub fn update_password(
        args: AuthArgs,
        notify: &Addr<NotifyActor>,
        user_id: Uuid,
        password: String,
        new_password: String,
        revoke_token_expires: i64,
    ) -> CoreResult<()> {
        // Update password requires token key type.
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalUpdatePassword,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalUpdatePassword,
            &user,
            KeyType::Token,
        )?;

        // User is allowed to update password in case `password_require_update` is true.

        // Check user password matches password hash.
        if let Err(err) = User::password_check(user.password_hash.as_ref().map(|x| &**x), &password)
        {
            args.audit.create_message(
                args.driver,
                AuditType::AuthLocalUpdatePassword,
                AuditMessage::PasswordNotSetOrIncorrect,
            )?;
            return Err(err);
        }

        // Successful update password, encode revoke token.
        let csrf = Auth::csrf_create_inner(args.driver, args.service, revoke_token_expires)?;
        let (revoke_token, _) = Jwt::encode_token_csrf(
            args.service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
            &csrf.key,
            &key.value,
            revoke_token_expires,
        )?;

        // Update user password, reread from driver.
        User::update_password(args.driver, Some(args.service), user.id, new_password)?;
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalUpdatePassword,
            user_id,
        )?;

        // Pass audit log to notification actor.
        let audit = args.audit.create_message(
            args.driver,
            AuditType::AuthLocalUpdatePassword,
            AuditMessage::UpdatePassword,
        )?;

        // Send update password action email.
        notify
            .try_send(EmailUpdatePassword::new(
                args.service.clone(),
                user,
                revoke_token,
                Some(audit),
            ))
            .map_err(|_err| CoreError::BadRequest)?;
        Ok(())
    }

    pub fn update_password_revoke(
        args: AuthArgs,
        token: String,
        audit_create: Option<AuditCreate2>,
    ) -> CoreResult<Option<Audit>> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(&token, args.service.id)?;

        // Update password revoke requires token key type.
        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthLocalUpdatePasswordRevoke,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthLocalUpdatePasswordRevoke,
            &user,
            KeyType::Token,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            args.service.id,
            user.id,
            JwtClaimsType::UpdatePasswordRevokeToken,
            &key.value,
            &token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                args.audit.create_message(
                    args.driver,
                    AuditType::AuthLocalUpdatePasswordRevoke,
                    AuditMessage::TokenInvalidOrExpired,
                )?;
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_verify_inner(
            args.driver,
            args.service,
            args.audit,
            csrf_key,
            AuditType::AuthLocalUpdatePasswordRevoke,
        )?;

        // Successful update password revoke, disable user and disable and revoke all keys associated with user.
        let update = UserUpdate {
            is_enabled: Some(false),
            name: None,
            locale: None,
            timezone: None,
            password_allow_reset: None,
            password_require_update: None,
        };
        User::update(args.driver, Some(args.service), user.id, &update)?;
        Key::update_many(
            args.driver,
            Some(args.service),
            user.id,
            Some(false),
            Some(true),
            None,
        )?;

        args.audit.create_message(
            args.driver,
            AuditType::AuthLocalUpdatePasswordRevoke,
            AuditMessage::UpdatePasswordRevoke,
        )?;
        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok(audit)
    }

    pub fn key_verify(
        args: AuthArgs,
        key: String,
        audit_create: Option<AuditCreate2>,
    ) -> CoreResult<(UserKey, Option<Audit>)> {
        // Key verify requires key key type.
        let key = Auth::key_read_by_user_value(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthKeyVerify,
            key,
            KeyType::Key,
        )?;

        // Check key is associated with user.
        let user = match key.user_id.ok_or_else(|| CoreError::BadRequest) {
            Ok(user_id) => Auth::user_read_by_id(
                args.driver,
                Some(args.service),
                args.audit,
                AuditType::AuthKeyVerify,
                user_id,
            ),
            Err(err) => {
                args.audit.create_message(
                    args.driver,
                    AuditType::AuthKeyVerify,
                    AuditMessage::KeyNotFound,
                )?;
                Err(err)
            }
        }?;

        // Successful key verify.
        let user_key = UserKey {
            user,
            key: key.value,
        };

        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok((user_key, audit))
    }

    pub fn key_revoke(
        args: AuthArgs,
        key: String,
        audit_create: Option<AuditCreate2>,
    ) -> CoreResult<Option<Audit>> {
        // Key revoke requires key key type.
        // Do not check key is enabled or not revoked.
        let key = Auth::key_read_by_user_value_unchecked(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthKeyRevoke,
            key,
            KeyType::Key,
        )?;

        // Successful key revoke, disable and revoke key.
        Key::update(
            args.driver,
            Some(args.service),
            key.id,
            Some(false),
            Some(true),
            None,
        )?;

        args.audit.create_message(
            args.driver,
            AuditType::AuthKeyRevoke,
            AuditMessage::KeyRevoke,
        )?;
        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok(audit)
    }

    pub fn token_verify(
        args: AuthArgs,
        token: String,
        audit_create: Option<AuditCreate2>,
    ) -> CoreResult<(UserTokenAccess, Option<Audit>)> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(&token, args.service.id)?;

        // Token verify requires token key type.
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthTokenVerify,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthTokenVerify,
            &user,
            KeyType::Token,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            args.service.id,
            user.id,
            JwtClaimsType::AccessToken,
            &key.value,
            &token,
        );
        let access_token_expires = match decoded {
            Ok((access_token_expires, _)) => access_token_expires,
            Err(err) => {
                args.audit.create_message(
                    args.driver,
                    AuditType::AuthTokenVerify,
                    AuditMessage::TokenInvalidOrExpired,
                )?;
                return Err(err);
            }
        };

        // Successful token verify.
        let user_token = UserTokenAccess {
            user,
            access_token: token.to_owned(),
            access_token_expires,
        };

        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok((user_token, audit))
    }

    pub fn token_refresh(
        args: AuthArgs,
        token: String,
        audit_create: Option<AuditCreate2>,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<(UserToken, Option<Audit>)> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, _) = Jwt::decode_unsafe(&token, args.service.id)?;

        // Token refresh requires token key type.
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthTokenRefresh,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthTokenRefresh,
            &user,
            KeyType::Token,
        )?;

        // Safely decode token with user key, this checks the type.
        let decoded = Jwt::decode_token(
            args.service.id,
            user.id,
            JwtClaimsType::RefreshToken,
            &key.value,
            &token,
        );
        let csrf_key = match decoded {
            Ok((_, csrf_key)) => csrf_key.ok_or_else(|| CoreError::BadRequest)?,
            Err(err) => {
                args.audit.create_message(
                    args.driver,
                    AuditType::AuthTokenRefresh,
                    AuditMessage::TokenInvalidOrExpired,
                )?;
                return Err(err);
            }
        };

        // Check the CSRF key to prevent reuse.
        Auth::csrf_verify_inner(
            args.driver,
            args.service,
            args.audit,
            csrf_key,
            AuditType::AuthTokenRefresh,
        )?;

        // Successful token refresh, encode user token.
        let user_token = Auth::encode_user_token(
            args.driver,
            args.service,
            args.audit,
            user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        args.audit.create_message(
            args.driver,
            AuditType::AuthTokenRefresh,
            AuditMessage::TokenRefresh,
        )?;
        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok((user_token, audit))
    }

    pub fn token_revoke(
        args: AuthArgs,
        token: String,
        audit_create: Option<AuditCreate2>,
    ) -> CoreResult<Option<Audit>> {
        // Unsafely decode token to get user identifier, used to read key for safe token decode.
        let (user_id, token_type) = Jwt::decode_unsafe(&token, args.service.id)?;

        // Token revoke requires token key type.
        // Do not check user, key is enabled or not revoked.
        let user = Auth::user_read_by_id_unchecked(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthTokenRevoke,
            user_id,
        )?;
        let key = Auth::key_read_by_user_unchecked(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthTokenRevoke,
            &user,
            KeyType::Token,
        )?;

        // Safely decode token with user key.
        let csrf_key =
            match Jwt::decode_token(args.service.id, user.id, token_type, &key.value, &token) {
                Ok((_, csrf_key)) => csrf_key,
                Err(err) => {
                    args.audit.create_message(
                        args.driver,
                        AuditType::AuthTokenRevoke,
                        AuditMessage::TokenInvalidOrExpired,
                    )?;
                    return Err(err);
                }
            };

        // If token has CSRF key, invalidate it now.
        if let Some(csrf_key) = csrf_key {
            Csrf::read_opt(args.driver, csrf_key)?;
        }

        // Successful token revoke, disable and revoke key associated with token.
        Key::update(
            args.driver,
            Some(args.service),
            key.id,
            Some(false),
            Some(true),
            None,
        )?;

        args.audit.create_message(
            args.driver,
            AuditType::AuthTokenRevoke,
            AuditMessage::TokenRevoke,
        )?;
        let audit = if let Some(audit_create) = audit_create {
            Some(args.audit.create(args.driver, audit_create)?)
        } else {
            None
        };
        Ok(audit)
    }

    /// TOTP code verification.
    pub fn totp(args: AuthArgs, user_id: Uuid, totp_code: String) -> CoreResult<()> {
        // TOTP requires token key type.
        let user = Auth::user_read_by_id(
            args.driver,
            Some(args.service),
            args.audit,
            AuditType::AuthTotp,
            user_id,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            args.service,
            args.audit,
            AuditType::AuthTotp,
            &user,
            KeyType::Totp,
        )?;
        let totp = TOTPBuilder::new()
            .base32_key(&key.value)
            .finalize()
            .map_err(CoreError::libreauth_oath)?;

        if !totp.is_valid(&totp_code) {
            args.audit.create_message(
                args.driver,
                AuditType::AuthTotp,
                AuditMessage::TotpInvalid,
            )?;
            Err(CoreError::BadRequest)
        } else {
            Ok(())
        }
    }

    /// CSRF creation.
    pub fn csrf_create(args: AuthArgs, expires_s: Option<i64>) -> CoreResult<Csrf> {
        let expires_s = expires_s.unwrap_or_else(Core::default_csrf_expires_s);
        Self::csrf_create_inner(args.driver, args.service, expires_s)
    }

    /// CSRF verification.
    pub fn csrf_verify(args: AuthArgs, csrf_key: String) -> CoreResult<()> {
        Self::csrf_verify_inner(
            args.driver,
            args.service,
            args.audit,
            csrf_key,
            AuditType::AuthCsrfVerify,
        )
    }

    /// OAuth2 user login.
    pub fn oauth2_login(
        args: AuthArgs,
        service_id: Uuid,
        email: String,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<(Service, UserToken)> {
        // Check service making url and callback requests match.
        if args.service.id != service_id {
            args.audit.create_message(
                args.driver,
                AuditType::AuthGithubOauth2Callback,
                AuditMessage::ServiceMismatch,
            )?;
            return Err(CoreError::BadRequest);
        }

        // OAuth2 login requires token key type.
        let service = Auth::service_read_by_id(
            args.driver,
            service_id,
            args.audit,
            AuditType::AuthGithubOauth2Callback,
        )?;
        let user = Auth::user_read_by_email(
            args.driver,
            Some(&service),
            args.audit,
            AuditType::AuthGithubOauth2Callback,
            email,
        )?;
        let key = Auth::key_read_by_user(
            args.driver,
            &service,
            args.audit,
            AuditType::AuthGithubOauth2Callback,
            &user,
            KeyType::Token,
        )?;

        // Successful OAuth2 login, return service for redirect callback integration.
        let user_token = Auth::encode_user_token(
            args.driver,
            &service,
            args.audit,
            user,
            &key,
            access_token_expires,
            refresh_token_expires,
        )?;

        args.audit.create_message(
            args.driver,
            AuditType::AuthOauth2Login,
            AuditMessage::Oauth2Login,
        )?;
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
                audit.service(Some(&service));
                if !service.is_enabled {
                    audit.create_message(driver, audit_type, AuditMessage::ServiceDisabled)?;
                    return Err(CoreError::BadRequest);
                }
                Ok(service)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::ServiceNotFound)?;
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
        let read = UserRead::Id(id);
        match User::read_opt(driver, service_mask, &read)?.ok_or_else(|| CoreError::BadRequest) {
            Ok(user) => {
                audit.user(Some(&user));
                if !user.is_enabled {
                    audit.create_message(driver, audit_type, AuditMessage::UserDisabled)?;
                    return Err(CoreError::BadRequest);
                }
                Ok(user)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::UserNotFound)?;
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
        let read = UserRead::Id(id);
        match User::read_opt(driver, service_mask, &read)?.ok_or_else(|| CoreError::BadRequest) {
            Ok(user) => {
                audit.user(Some(&user));
                Ok(user)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::UserNotFound)?;
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
        email: String,
    ) -> CoreResult<User> {
        let read = UserRead::Email(email);
        match User::read_opt(driver, service_mask, &read)?.ok_or_else(|| CoreError::BadRequest) {
            Ok(user) => {
                audit.user(Some(&user));
                if !user.is_enabled {
                    audit.create_message(driver, audit_type, AuditMessage::UserDisabled)?;
                    return Err(CoreError::BadRequest);
                }
                Ok(user)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::UserNotFound)?;
                Err(err)
            }
        }
    }

    /// Read key by user reference and key type.
    /// Also checks key is enabled and not revoked, returns bad request if disabled.
    fn key_read_by_user(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        audit_type: AuditType,
        user: &User,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        match Key::read_by_user(driver, &service, &user, key_type)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_message(driver, audit_type, AuditMessage::KeyDisabledOrRevoked)?;
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::KeyNotFound)?;
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
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        match Key::read_by_user(driver, &service, &user, key_type)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.user_key(Some(&key));
                Ok(key)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::KeyNotFound)?;
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
        key: String,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        match Key::read_by_user_value(driver, service, key, key_type)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.user_key(Some(&key));
                if !key.is_enabled || key.is_revoked {
                    audit.create_message(driver, audit_type, AuditMessage::KeyDisabledOrRevoked)?;
                    return Err(CoreError::BadRequest);
                }
                Ok(key)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::KeyNotFound)?;
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
        key: String,
        key_type: KeyType,
    ) -> CoreResult<KeyWithValue> {
        match Key::read_by_user_value(driver, service, key, key_type)?
            .ok_or_else(|| CoreError::BadRequest)
        {
            Ok(key) => {
                audit.user_key(Some(&key));
                Ok(key)
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::KeyNotFound)?;
                Err(err)
            }
        }
    }

    /// Build user token by encoding access and refresh tokens.
    fn encode_user_token(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        user: User,
        key: &KeyWithValue,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> CoreResult<UserToken> {
        let csrf = Auth::csrf_create_inner(driver, service, refresh_token_expires)?;
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
            user,
            access_token,
            access_token_expires,
            refresh_token,
            refresh_token_expires,
        })
    }

    /// Create a new CSRF key, value pair using random key.
    fn csrf_create_inner(
        driver: &dyn Driver,
        service: &Service,
        token_expires: i64,
    ) -> CoreResult<Csrf> {
        let csrf_key = Key::value_generate();
        Csrf::create(driver, service, csrf_key.clone(), csrf_key, token_expires)
    }

    /// Verify a CSRF key is valid by reading it, this will also delete the key.
    /// Also checks service verifying CSRF is same service that created it.
    fn csrf_verify_inner(
        driver: &dyn Driver,
        service: &Service,
        audit: &mut AuditBuilder,
        csrf_key: String,
        audit_type: AuditType,
    ) -> CoreResult<()> {
        let res = Csrf::read_opt(driver, csrf_key)?.ok_or_else(|| CoreError::BadRequest);

        match res {
            Ok(csrf) => {
                if csrf.service_id != service.id {
                    audit.create_message(driver, audit_type, AuditMessage::CsrfNotFoundOrUsed)?;
                    return Err(CoreError::BadRequest);
                }
                Ok(())
            }
            Err(err) => {
                audit.create_message(driver, audit_type, AuditMessage::CsrfNotFoundOrUsed)?;
                Err(err)
            }
        }
    }
}
