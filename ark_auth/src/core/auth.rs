use crate::core;
use crate::core::audit::{AuditBuilder, AuditMessage, AuditPath};
use crate::core::{Csrf, Error, Key, Service, User, UserKey, UserToken, UserTokenPartial};
use crate::driver::Driver;

/// User authentication using email address and password.
pub fn login(
    driver: &Driver,
    service: &Service,
    mut audit: AuditBuilder,
    email: &str,
    password: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    // Get user using email, check is enabled.
    let user = match user_read_by_email(driver, Some(service), &mut audit, email) {
        Ok(user) => user,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::LoginError(AuditMessage::UserNotFoundOrDisabled.into()),
            );
            return Err(err);
        }
    };

    // Get key with user, check is enabled and not revoked.
    let key = match key_read_by_user(driver, service, &mut audit, &user) {
        Ok(key) => key,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::LoginError(AuditMessage::KeyNotFoundOrDisabled.into()),
            );
            return Err(err);
        }
    };

    // Check user password match.
    if let Err(err) = core::check_password(user.password_hash.as_ref().map(|x| &**x), &password) {
        audit.create(
            driver,
            AuditPath::LoginError(AuditMessage::PasswordNotSetOrIncorrect.into()),
        );
        return Err(err);
    }

    // Successful login.
    let user_token = encode_user_token(
        driver,
        &service,
        &user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )?;
    audit.create(driver, AuditPath::Login(AuditMessage::Login.into()));
    Ok(user_token)
}

/// User reset password request.
pub fn reset_password(
    driver: &Driver,
    service: &Service,
    mut audit: AuditBuilder,
    email: &str,
    token_expires: i64,
) -> Result<(User, String), Error> {
    // Get user using email, check is enabled.
    let user = match user_read_by_email(driver, Some(service), &mut audit, email) {
        Ok(user) => user,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::ResetPasswordError(AuditMessage::UserNotFoundOrDisabled.into()),
            );
            return Err(err);
        }
    };

    // Get key with user, check is enabled and not revoked.
    let key = match key_read_by_user(driver, service, &mut audit, &user) {
        Ok(key) => key,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::ResetPasswordError(AuditMessage::KeyNotFoundOrDisabled.into()),
            );
            return Err(err);
        }
    };

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
    audit.create(
        driver,
        AuditPath::ResetPassword(AuditMessage::ResetPassword.into()),
    );
    Ok((user, token))
}

/// User reset password confirm.
pub fn reset_password_confirm(
    driver: &Driver,
    service: &Service,
    mut audit: AuditBuilder,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user using email, check is enabled.
    let user = match user_read_by_id(driver, Some(service), &mut audit, &user_id) {
        Ok(user) => user,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::ResetPasswordConfirmError(AuditMessage::UserNotFoundOrDisabled.into()),
            );
            return Err(err);
        }
    };

    // Get key with user, check is enabled and not revoked.
    let key = match key_read_by_user(driver, service, &mut audit, &user) {
        Ok(key) => key,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::ResetPasswordConfirmError(AuditMessage::KeyNotFoundOrDisabled.into()),
            );
            return Err(err);
        }
    };

    // Safely decode token with user key, this checks the type.
    let decoded = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::ResetPasswordToken,
        &key.value,
        token,
    );
    let csrf_key = match decoded {
        Ok((_, csrf_key)) => {
            let csrf_key = csrf_key.ok_or_else(|| Error::BadRequest)?;
            csrf_key
        }
        Err(err) => {
            audit.create(
                driver,
                AuditPath::ResetPasswordConfirmError(AuditMessage::TokenInvalidOrExpired.into()),
            );
            return Err(err);
        }
    };

    // Check the CSRF key to prevent reuse.
    let _ = match csrf_check(driver, &csrf_key) {
        Ok(checked) => checked,
        Err(err) => {
            audit.create(
                driver,
                AuditPath::ResetPasswordConfirmError(AuditMessage::CsrfNotFoundOrUsed.into()),
            );
            return Err(err);
        }
    };

    // Sucessful reset password confirm, update user password.
    let count = core::user::update_password_by_id(driver, Some(service), &user.id, password)?;
    audit.create(
        driver,
        AuditPath::ResetPasswordConfirm(AuditMessage::ResetPasswordConfirm.into()),
    );
    Ok(count)
}

/// User update email request.
pub fn update_email(
    driver: &Driver,
    service: &Service,
    audit: AuditBuilder,
    key: Option<&str>,
    token: Option<&str>,
    password: &str,
    new_email: &str,
    revoke_token_expires: i64,
) -> Result<(User, String, String), Error> {
    // Get user and key using verified token or key, check is enabled/not revoked and password match.
    let (user_id, mut audit) = key_or_token_verify(driver, service, audit, key, token)?;
    let user = user_read_by_id(driver, Some(service), &mut audit, &user_id)?;
    let key = key_read_by_user(driver, service, &mut audit, &user)?;
    core::check_password(user.password_hash.as_ref().map(|x| &**x), &password)?;
    let old_email = user.email.to_owned();

    // Encode revoke token.
    let csrf = csrf_create(driver, service, revoke_token_expires)?;
    let (revoke_token, _) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdateEmailRevokeToken,
        Some(&csrf.key),
        &key.value,
        revoke_token_expires,
    )?;

    // Update user email and reread from driver.
    core::user::update_email_by_id(driver, Some(service), &user.id, new_email)?;
    let user = user_read_by_id(driver, Some(service), &mut audit, &user_id)?;
    Ok((user, old_email, revoke_token))
}

/// User update email revoke request.
pub fn update_email_revoke(
    driver: &Driver,
    service: &Service,
    _audit: AuditBuilder,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, do not check is enabled/not revoked.
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;

    // Safely decode token with user key, check type and csrf value.
    let (_, csrf_key) = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdateEmailRevokeToken,
        &key.value,
        token,
    )?;
    let csrf_key = csrf_key.ok_or_else(|| Error::BadRequest)?;
    csrf_check(driver, &csrf_key)?;

    // Disable user and disable and revoke all keys associated with user.
    core::user::update_by_id(driver, Some(service), &user.id, Some(false), None)?;
    let count = core::key::update_many_by_user_id(
        driver,
        Some(service),
        &user.id,
        Some(false),
        Some(true),
        None,
    )?;
    Ok(count + 1)
}

/// User update password request.
pub fn update_password(
    driver: &Driver,
    service: &Service,
    audit: AuditBuilder,
    key: Option<&str>,
    token: Option<&str>,
    password: &str,
    new_password: &str,
    revoke_token_expires: i64,
) -> Result<(User, String), Error> {
    // Get user and key using verified token or key, check is enabled/not revoked and password match.
    let (user_id, mut audit) = key_or_token_verify(driver, service, audit, key, token)?;
    let user = user_read_by_id(driver, Some(service), &mut audit, &user_id)?;
    let key = key_read_by_user(driver, service, &mut audit, &user)?;
    core::check_password(user.password_hash.as_ref().map(|x| &**x), &password)?;

    // Encode revoke token.
    let csrf = csrf_create(driver, service, revoke_token_expires)?;
    let (revoke_token, _) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdatePasswordRevokeToken,
        Some(&csrf.key),
        &key.value,
        revoke_token_expires,
    )?;

    // Update user password and reread from driver.
    core::user::update_password_by_id(driver, Some(service), &user.id, new_password)?;
    let user = user_read_by_id(driver, Some(service), &mut audit, &user_id)?;
    Ok((user, revoke_token))
}

/// User update password revoke request.
pub fn update_password_revoke(
    driver: &Driver,
    service: &Service,
    _audit: AuditBuilder,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, do not check is enabled/not revoked.
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;

    // Safely decode token with user key, check type and csrf value.
    let (_, csrf_key) = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdatePasswordRevokeToken,
        &key.value,
        token,
    )?;
    let csrf_key = csrf_key.ok_or_else(|| Error::BadRequest)?;
    csrf_check(driver, &csrf_key)?;

    // Disable user and disable and revoke all keys associated with user.
    core::user::update_by_id(driver, Some(service), &user.id, Some(false), None)?;
    let count = core::key::update_many_by_user_id(
        driver,
        Some(service),
        &user.id,
        Some(false),
        Some(true),
        None,
    )?;
    Ok(count + 1)
}

/// Verify user key.
pub fn key_verify(
    driver: &Driver,
    service: &Service,
    mut audit: AuditBuilder,
    key: &str,
) -> Result<(UserKey, AuditBuilder), Error> {
    // Get key, check is enabled/not revoked and associated with user.
    let key = key_read_by_user_value(driver, service, &mut audit, key)?;
    let user_id = key.user_id.ok_or_else(|| Error::BadRequest)?;

    // Return key.
    let user_key = UserKey {
        user_id,
        key: key.value,
    };
    Ok((user_key, audit))
}

/// Revoke user key.
pub fn key_revoke(
    driver: &Driver,
    service: &Service,
    _audit: AuditBuilder,
    key: &str,
) -> Result<usize, Error> {
    // Get key, do not check is enabled/not revoked.
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;

    // Disable and revoke key.
    core::key::update_by_id(
        driver,
        Some(service),
        &key.id,
        Some(false),
        Some(true),
        None,
    )?;
    Ok(1)
}

/// Verify user token.
pub fn token_verify(
    driver: &Driver,
    service: &Service,
    mut audit: AuditBuilder,
    token: &str,
) -> Result<(UserTokenPartial, AuditBuilder), Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, check is enabled/not revoked.
    let user = user_read_by_id(driver, Some(service), &mut audit, &user_id)?;
    let key = key_read_by_user(driver, service, &mut audit, &user)?;

    // Safely decode token with user key, check type.
    let (access_token_expires, _) = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::AccessToken,
        &key.value,
        token,
    )?;

    // Return partial token.
    let user_token = UserTokenPartial {
        user_id: user.id.to_owned(),
        access_token: token.to_owned(),
        access_token_expires,
    };
    Ok((user_token, audit))
}

/// Refresh token.
pub fn token_refresh(
    driver: &Driver,
    service: &Service,
    mut audit: AuditBuilder,
    token: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, check is enabled/not revoked.
    let user = user_read_by_id(driver, Some(service), &mut audit, &user_id)?;
    let key = key_read_by_user(driver, service, &mut audit, &user)?;

    // Safely decode token with user key, check type and csrf value.
    let (_, csrf_key) = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::RefreshToken,
        &key.value,
        token,
    )?;
    let csrf_key = csrf_key.ok_or_else(|| Error::BadRequest)?;
    csrf_check(driver, &csrf_key)?;

    // Encode user token containing new access token and refresh token.
    encode_user_token(
        driver,
        &service,
        &user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )
}

/// Revoke token.
pub fn token_revoke(
    driver: &Driver,
    service: &Service,
    _audit: AuditBuilder,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, token_type) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, do not check is enabled/not revoked.
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;

    // Safely decode token with user key, if it has CSRF key, invalidate it now.
    let (_, token_csrf) =
        core::jwt::decode_token(&service.id, &user.id, token_type, &key.value, token)?;
    if let Some(token_csrf) = token_csrf {
        core::csrf::read_by_key(driver, &token_csrf)?;
    }
    // Disable and revoke key associated with token.
    core::key::update_by_id(
        driver,
        Some(service),
        &key.id,
        Some(false),
        Some(true),
        None,
    )?;
    Ok(1)
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &Driver,
    service_id: &str,
    mut audit: AuditBuilder,
    email: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<(Service, UserToken), Error> {
    // Get service, user and key, check is enabled/not revoked.
    let service = service_read_by_id(driver, service_id)?;
    let user = user_read_by_email(driver, Some(&service), &mut audit, email)?;
    let key = key_read_by_user(driver, &service, &mut audit, &user)?;

    // Encode user token containing access token and refresh token.
    let user_token = encode_user_token(
        driver,
        &service,
        &user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )?;

    // Return service for redirect callback integration.
    Ok((service, user_token))
}

/// Read service by ID.
/// Also checks service is enabled, returns bad request if disabled.
fn service_read_by_id(driver: &Driver, service_id: &str) -> Result<Service, Error> {
    let service = driver
        .service_read_by_id(service_id)
        .map_err(Error::Driver)?
        .ok_or_else(|| Error::BadRequest)?;
    if !service.is_enabled {
        return Err(Error::BadRequest);
    }
    Ok(service)
}

/// Read user by ID.
/// Checks user is enabled, returns bad request if disabled.
fn user_read_by_id(
    driver: &Driver,
    service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    id: &str,
) -> Result<User, Error> {
    let user =
        core::user::read_by_id(driver, service_mask, id)?.ok_or_else(|| Error::BadRequest)?;

    audit.set_user(Some(&user));
    if !user.is_enabled {
        return Err(Error::BadRequest);
    }
    Ok(user)
}

/// Read user by email address.
/// Also checks user is enabled, returns bad request if disabled.
fn user_read_by_email(
    driver: &Driver,
    service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    email: &str,
) -> Result<User, Error> {
    let user =
        core::user::read_by_email(driver, service_mask, email)?.ok_or_else(|| Error::BadRequest)?;

    audit.set_user(Some(&user));
    if !user.is_enabled {
        return Err(Error::BadRequest);
    }
    Ok(user)
}

/// Read key by user reference.
/// Also checks key is enabled and not revoked, returns bad request if disabled.
fn key_read_by_user(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    user: &User,
) -> Result<Key, Error> {
    let key = core::key::read_by_user(driver, &service, &user)?.ok_or_else(|| Error::BadRequest)?;

    audit.set_user_key(Some(&key));
    if !key.is_enabled || key.is_revoked {
        return Err(Error::BadRequest);
    }
    Ok(key)
}

/// Read key by user value.
/// Also checks key is enabled and not revoked, returns bad request if disabled.
fn key_read_by_user_value(
    driver: &Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    key: &str,
) -> Result<Key, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;

    audit.set_user_key(Some(&key));
    if !key.is_enabled || key.is_revoked {
        return Err(Error::BadRequest);
    }
    Ok(key)
}

/// Get user ID from valid key or token.
fn key_or_token_verify(
    driver: &Driver,
    service: &Service,
    audit: AuditBuilder,
    key: Option<&str>,
    token: Option<&str>,
) -> Result<(String, AuditBuilder), Error> {
    match key {
        Some(key) => {
            let (user_key, audit) = key_verify(driver, service, audit, key)?;
            Ok((user_key.user_id, audit))
        }
        None => match token {
            Some(token) => {
                let (user_token, audit) = token_verify(driver, service, audit, token)?;
                Ok((user_token.user_id, audit))
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
fn csrf_check(driver: &Driver, csrf_key: &str) -> Result<(), Error> {
    core::csrf::read_by_key(driver, csrf_key)?
        .ok_or_else(|| Error::BadRequest)
        .map(|_| ())
}
