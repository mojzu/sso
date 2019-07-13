use crate::core;
use crate::core::audit::{AuditBuilder, AuditMessage, AuditPath};
use crate::core::{Csrf, Error, Key, Service, User, UserKey, UserToken, UserTokenPartial};
use crate::driver::Driver;
use crate::notify::{EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword, NotifyExecutor};
use actix::Addr;

pub fn login(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    email: &str,
    password: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    let user = user_read_by_email(driver, Some(service), audit, AuditPath::LoginError, email)?;
    let key = key_read_by_user(driver, service, audit, AuditPath::LoginError, &user)?;

    // Check user password matches password hash.
    if let Err(err) = core::check_password(user.password_hash.as_ref().map(|x| &**x), &password) {
        audit.create_internal(
            driver,
            AuditPath::LoginError,
            AuditMessage::PasswordNotSetOrIncorrect,
        );
        return Err(err);
    }

    // Successful login, encode and return user token.
    let user_token = encode_user_token(
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

pub fn reset_password(
    driver: &Driver,
    notify: &Addr<NotifyExecutor>,
    service: &Service,
    audit: &mut AuditBuilder,
    email: &str,
    token_expires: i64,
) -> Result<(), Error> {
    let user = user_read_by_email(
        driver,
        Some(service),
        audit,
        AuditPath::ResetPasswordError,
        email,
    )?;
    let key = key_read_by_user(driver, service, audit, AuditPath::ResetPasswordError, &user)?;

    // Successful reset password, encode reset token.
    let csrf = csrf_create(driver, service, token_expires)?;
    let (token, _) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::ResetPasswordToken,
        Some(&csrf.key),
        &key.value,
        token_expires,
    )?;

    // Send reset password action email.
    notify
        .try_send(EmailResetPassword {
            service: service.clone(),
            user,
            token,
        })
        .map_err(|_err| Error::BadRequest)?;

    audit.create_internal(
        driver,
        AuditPath::ResetPassword,
        AuditMessage::ResetPassword,
    );
    Ok(())
}

pub fn reset_password_confirm(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::ResetPasswordConfirmError,
        &user_id,
    )?;
    let key = key_read_by_user(
        driver,
        service,
        audit,
        AuditPath::ResetPasswordConfirmError,
        &user,
    )?;

    // Safely decode token with user key, this checks the type.
    let decoded = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::ResetPasswordToken,
        &key.value,
        token,
    );
    let csrf_key = match decoded {
        Ok((_, csrf_key)) => csrf_key.ok_or_else(|| Error::BadRequest)?,
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
    csrf_check(
        driver,
        &csrf_key,
        &audit,
        AuditPath::ResetPasswordConfirmError,
    )?;

    // Sucessful reset password confirm, update user password.
    let count =
        core::user::update_password_by_id(driver, Some(service), audit, &user.id, password)?;

    audit.create_internal(
        driver,
        AuditPath::ResetPasswordConfirm,
        AuditMessage::ResetPasswordConfirm,
    );
    Ok(count)
}

pub fn update_email(
    driver: &Driver,
    notify: &Addr<NotifyExecutor>,
    service: &Service,
    audit: &mut AuditBuilder,
    key: Option<&str>,
    token: Option<&str>,
    password: &str,
    new_email: &str,
    revoke_token_expires: i64,
) -> Result<(), Error> {
    // Verify key or token argument to get user ID.
    let user_id = key_or_token_verify(driver, service, audit, key, token)?;

    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::UpdateEmailError,
        &user_id,
    )?;
    let key = key_read_by_user(driver, service, audit, AuditPath::UpdateEmailError, &user)?;
    let old_email = user.email.to_owned();

    // Check user password matches password hash.
    if let Err(err) = core::check_password(user.password_hash.as_ref().map(|x| &**x), &password) {
        audit.create_internal(
            driver,
            AuditPath::UpdateEmailError,
            AuditMessage::PasswordNotSetOrIncorrect,
        );
        return Err(err);
    }

    // Successful update email, encode revoke token.
    let csrf = csrf_create(driver, service, revoke_token_expires)?;
    let (revoke_token, _) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdateEmailRevokeToken,
        Some(&csrf.key),
        &key.value,
        revoke_token_expires,
    )?;

    // Update user email.
    core::user::update_email_by_id(driver, Some(service), audit, &user.id, new_email)?;
    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::UpdateEmailError,
        &user_id,
    )?;

    // Send update email action email.
    notify
        .try_send(EmailUpdateEmail {
            service: service.clone(),
            user,
            old_email,
            token: revoke_token,
        })
        .map_err(|_err| Error::BadRequest)?;

    audit.create_internal(driver, AuditPath::UpdateEmail, AuditMessage::UpdateEmail);
    Ok(())
}

pub fn update_email_revoke(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Do not check user, key is enabled or not revoked.
    let user = user_read_by_id_unchecked(
        driver,
        Some(service),
        audit,
        AuditPath::UpdateEmailRevokeError,
        &user_id,
    )?;
    let key = key_read_by_user_unchecked(
        driver,
        service,
        audit,
        AuditPath::UpdateEmailRevokeError,
        &user,
    )?;

    // Safely decode token with user key, this checks the type.
    let decoded = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdateEmailRevokeToken,
        &key.value,
        token,
    );
    let csrf_key = match decoded {
        Ok((_, csrf_key)) => csrf_key.ok_or_else(|| Error::BadRequest)?,
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
    csrf_check(driver, &csrf_key, &audit, AuditPath::UpdateEmailRevokeError)?;

    // Successful update email revoke, disable user and disable and revoke all keys associated with user.
    core::user::update_by_id(driver, Some(service), audit, &user.id, Some(false), None)?;
    let count = core::key::update_many_by_user_id(
        driver,
        Some(service),
        audit,
        &user.id,
        Some(false),
        Some(true),
        None,
    )?;

    audit.create_internal(
        driver,
        AuditPath::UpdateEmailRevoke,
        AuditMessage::UpdateEmailRevoke,
    );
    Ok(count + 1)
}

pub fn update_password(
    driver: &Driver,
    notify: &Addr<NotifyExecutor>,
    service: &Service,
    audit: &mut AuditBuilder,
    key: Option<&str>,
    token: Option<&str>,
    password: &str,
    new_password: &str,
    revoke_token_expires: i64,
) -> Result<(), Error> {
    // Verify key or token argument to get user ID.
    let user_id = key_or_token_verify(driver, service, audit, key, token)?;

    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::UpdatePasswordError,
        &user_id,
    )?;
    let key = key_read_by_user(
        driver,
        service,
        audit,
        AuditPath::UpdatePasswordError,
        &user,
    )?;

    // Check user password matches password hash.
    if let Err(err) = core::check_password(user.password_hash.as_ref().map(|x| &**x), &password) {
        audit.create_internal(
            driver,
            AuditPath::UpdatePasswordError,
            AuditMessage::PasswordNotSetOrIncorrect,
        );
        return Err(err);
    }

    // Successful update password, encode revoke token.
    let csrf = csrf_create(driver, service, revoke_token_expires)?;
    let (revoke_token, _) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdatePasswordRevokeToken,
        Some(&csrf.key),
        &key.value,
        revoke_token_expires,
    )?;

    // Update user password, reread from driver.
    core::user::update_password_by_id(driver, Some(service), audit, &user.id, new_password)?;
    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::UpdatePasswordError,
        &user_id,
    )?;

    // Send update password action email.
    notify
        .try_send(EmailUpdatePassword {
            service: service.clone(),
            user,
            token: revoke_token,
        })
        .map_err(|_err| Error::BadRequest)?;

    audit.create_internal(
        driver,
        AuditPath::UpdatePassword,
        AuditMessage::UpdatePassword,
    );
    Ok(())
}

pub fn update_password_revoke(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Do not check user, key is enabled or not revoked.
    let user = user_read_by_id_unchecked(
        driver,
        Some(service),
        audit,
        AuditPath::UpdatePasswordRevokeError,
        &user_id,
    )?;
    let key = key_read_by_user_unchecked(
        driver,
        service,
        audit,
        AuditPath::UpdatePasswordRevokeError,
        &user,
    )?;

    // Safely decode token with user key, this checks the type.
    let decoded = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdatePasswordRevokeToken,
        &key.value,
        token,
    );
    let csrf_key = match decoded {
        Ok((_, csrf_key)) => csrf_key.ok_or_else(|| Error::BadRequest)?,
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
    csrf_check(
        driver,
        &csrf_key,
        &audit,
        AuditPath::UpdatePasswordRevokeError,
    )?;

    // Successful update password revoke, disable user and disable and revoke all keys associated with user.
    core::user::update_by_id(driver, Some(service), audit, &user.id, Some(false), None)?;
    let count = core::key::update_many_by_user_id(
        driver,
        Some(service),
        audit,
        &user.id,
        Some(false),
        Some(true),
        None,
    )?;

    audit.create_internal(
        driver,
        AuditPath::UpdatePasswordRevoke,
        AuditMessage::UpdatePasswordRevoke,
    );
    Ok(count + 1)
}

pub fn key_verify(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    key: &str,
) -> Result<UserKey, Error> {
    let key = key_read_by_user_value(driver, service, audit, AuditPath::KeyVerifyError, key)?;

    // Check key is associated with user.
    let user_id = match key.user_id.ok_or_else(|| Error::BadRequest) {
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
    Ok(user_key)
}

pub fn key_revoke(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    key: &str,
) -> Result<usize, Error> {
    // Do not check key is enabled or not revoked.
    let key =
        key_read_by_user_value_unchecked(driver, service, audit, AuditPath::KeyRevokeError, key)?;

    // Successful key revoke, disable and revoke key.
    core::key::update_by_id(
        driver,
        Some(service),
        audit,
        &key.id,
        Some(false),
        Some(true),
        None,
    )?;

    audit.create_internal(driver, AuditPath::KeyRevoke, AuditMessage::KeyRevoke);
    Ok(1)
}

pub fn token_verify(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    token: &str,
) -> Result<UserTokenPartial, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::TokenVerifyError,
        &user_id,
    )?;
    let key = key_read_by_user(driver, service, audit, AuditPath::TokenVerifyError, &user)?;

    // Safely decode token with user key, this checks the type.
    let decoded = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::AccessToken,
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
    let user_token = UserTokenPartial {
        user_id: user.id.to_owned(),
        access_token: token.to_owned(),
        access_token_expires,
    };
    Ok(user_token)
}

pub fn token_refresh(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    token: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    let user = user_read_by_id(
        driver,
        Some(service),
        audit,
        AuditPath::TokenRefreshError,
        &user_id,
    )?;
    let key = key_read_by_user(driver, service, audit, AuditPath::TokenRefreshError, &user)?;

    // Safely decode token with user key, this checks the type.
    let decoded = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::RefreshToken,
        &key.value,
        token,
    );
    let csrf_key = match decoded {
        Ok((_, csrf_key)) => csrf_key.ok_or_else(|| Error::BadRequest)?,
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
    csrf_check(driver, &csrf_key, &audit, AuditPath::TokenRefreshError)?;

    // Successful token refresh, encode user token.
    let user_token = encode_user_token(
        driver,
        &service,
        &user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )?;

    audit.create_internal(driver, AuditPath::TokenRefresh, AuditMessage::TokenRefresh);
    Ok(user_token)
}

pub fn token_revoke(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, token_type) = core::jwt::decode_unsafe(token, &service.id)?;

    // Do not check user, key is enabled or not revoked.
    let user = user_read_by_id_unchecked(
        driver,
        Some(service),
        audit,
        AuditPath::TokenRevokeError,
        &user_id,
    )?;
    let key =
        key_read_by_user_unchecked(driver, service, audit, AuditPath::TokenRevokeError, &user)?;

    // Safely decode token with user key.
    let csrf_key =
        match core::jwt::decode_token(&service.id, &user.id, token_type, &key.value, token) {
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
        core::csrf::read_by_key(driver, &csrf_key)?;
    }

    // Successful token revoke, disable and revoke key associated with token.
    core::key::update_by_id(
        driver,
        Some(service),
        audit,
        &key.id,
        Some(false),
        Some(true),
        None,
    )?;

    audit.create_internal(driver, AuditPath::TokenRevoke, AuditMessage::TokenRevoke);
    Ok(1)
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &Driver,
    service_id: &str,
    audit: &mut AuditBuilder,
    email: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<(Service, UserToken), Error> {
    let service = service_read_by_id(driver, service_id, audit, AuditPath::Oauth2LoginError)?;
    let user = user_read_by_email(
        driver,
        Some(&service),
        audit,
        AuditPath::Oauth2LoginError,
        email,
    )?;
    let key = key_read_by_user(driver, &service, audit, AuditPath::Oauth2LoginError, &user)?;

    // Successful OAuth2 login, return service for redirect callback integration.
    let user_token = encode_user_token(
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
    driver: &Driver,
    service_id: &str,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
) -> Result<Service, Error> {
    match driver
        .service_read_by_id(service_id)
        .map_err(Error::Driver)?
        .ok_or_else(|| Error::BadRequest)
    {
        Ok(service) => {
            audit.set_service(Some(&service));
            if !service.is_enabled {
                audit.create_internal(driver, audit_path, AuditMessage::ServiceDisabled);
                return Err(Error::BadRequest);
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
    driver: &Driver,
    service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    id: &str,
) -> Result<User, Error> {
    match core::user::read_by_id(driver, service_mask, audit, id)?.ok_or_else(|| Error::BadRequest)
    {
        Ok(user) => {
            audit.set_user(Some(&user));
            if !user.is_enabled {
                audit.create_internal(driver, audit_path, AuditMessage::UserDisabled);
                return Err(Error::BadRequest);
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
    driver: &Driver,
    service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    id: &str,
) -> Result<User, Error> {
    match core::user::read_by_id(driver, service_mask, audit, id)?.ok_or_else(|| Error::BadRequest)
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
    driver: &Driver,
    service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    email: &str,
) -> Result<User, Error> {
    match core::user::read_by_email(driver, service_mask, audit, email)?
        .ok_or_else(|| Error::BadRequest)
    {
        Ok(user) => {
            audit.set_user(Some(&user));
            if !user.is_enabled {
                audit.create_internal(driver, audit_path, AuditMessage::UserDisabled);
                return Err(Error::BadRequest);
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
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    user: &User,
) -> Result<Key, Error> {
    match core::key::read_by_user(driver, &service, audit, &user)?.ok_or_else(|| Error::BadRequest)
    {
        Ok(key) => {
            audit.set_user_key(Some(&key));
            if !key.is_enabled || key.is_revoked {
                audit.create_internal(driver, audit_path, AuditMessage::KeyDisabledOrRevoked);
                return Err(Error::BadRequest);
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
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    user: &User,
) -> Result<Key, Error> {
    match core::key::read_by_user(driver, &service, audit, &user)?.ok_or_else(|| Error::BadRequest)
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
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    key: &str,
) -> Result<Key, Error> {
    match core::key::read_by_user_value(driver, service, audit, key)?
        .ok_or_else(|| Error::BadRequest)
    {
        Ok(key) => {
            audit.set_user_key(Some(&key));
            if !key.is_enabled || key.is_revoked {
                audit.create_internal(driver, audit_path, AuditMessage::KeyDisabledOrRevoked);
                return Err(Error::BadRequest);
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
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    audit_path: AuditPath,
    key: &str,
) -> Result<Key, Error> {
    match core::key::read_by_user_value(driver, service, audit, key)?
        .ok_or_else(|| Error::BadRequest)
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
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    key: Option<&str>,
    token: Option<&str>,
) -> Result<String, Error> {
    match key {
        Some(key) => {
            let user_key = key_verify(driver, service, audit, key)?;
            Ok(user_key.user_id)
        }
        None => match token {
            Some(token) => {
                let user_token = token_verify(driver, service, audit, token)?;
                Ok(user_token.user_id)
            }
            None => Err(Error::Forbidden),
        },
    }
}

/// Build user token by encoding access and refresh tokens.
fn encode_user_token(
    driver: &Driver,
    service: &Service,
    user: &User,
    key: &Key,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    let csrf = csrf_create(driver, &service, refresh_token_expires)?;
    let (access_token, access_token_expires) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::AccessToken,
        None,
        &key.value,
        access_token_expires,
    )?;
    let (refresh_token, refresh_token_expires) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::RefreshToken,
        Some(&csrf.key),
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
fn csrf_create(driver: &Driver, service: &Service, token_expires: i64) -> Result<Csrf, Error> {
    let csrf_key = uuid::Uuid::new_v4().to_simple().to_string();
    core::csrf::create(driver, service, &csrf_key, &csrf_key, token_expires)
}

/// Check a CSRF key is valid by reading it, this will also delete the key.
fn csrf_check(
    driver: &Driver,
    csrf_key: &str,
    audit: &AuditBuilder,
    audit_path: AuditPath,
) -> Result<(), Error> {
    let res = core::csrf::read_by_key(driver, csrf_key)?
        .ok_or_else(|| Error::BadRequest)
        .map(|_| ());

    match res {
        Ok(_) => Ok(()),
        Err(err) => {
            audit.create_internal(driver, audit_path, AuditMessage::CsrfNotFoundOrUsed);
            Err(err)
        }
    }
}
