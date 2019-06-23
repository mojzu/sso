use crate::core;
use crate::core::{Csrf, Error, Key, Service, User, UserKey, UserToken, UserTokenPartial};
use crate::driver::Driver;

// TODO(feature): Warning logs for bad requests.

/// User authentication using email address and password.
pub fn login(
    driver: &Driver,
    service: &Service,
    email: &str,
    password: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    // Get user and key using email, check is active and password match.
    let user = user_read_by_email(driver, Some(service), email)?;
    let key = key_read_by_user(driver, service, &user)?;
    core::check_password(user.password_hash.as_ref().map(|x| &**x), &password)?;

    // Encode user token containing access token and refresh token.
    let user_token = encode_user_token(
        driver,
        &service,
        &user,
        &key,
        access_token_expires,
        refresh_token_expires,
    )?;
    Ok(user_token)
}

/// User reset password request.
pub fn reset_password(
    driver: &Driver,
    service: &Service,
    email: &str,
    token_expires: i64,
) -> Result<(User, String), Error> {
    // Get user and key using email, check is active.
    let user = user_read_by_email(driver, Some(service), email)?;
    let key = key_read_by_user(driver, service, &user)?;

    // Encode token.
    let csrf = csrf_create(driver, service, token_expires)?;
    let (token, _) = core::jwt::encode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::UpdatePasswordRevokeToken,
        Some(&csrf.key),
        &key.value,
        token_expires,
    )?;

    // Return user and token.
    Ok((user, token))
}

/// User reset password confirm.
pub fn reset_password_confirm(
    driver: &Driver,
    service: &Service,
    token: &str,
    password: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, check is active.
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    let key = key_read_by_user(driver, service, &user)?;

    // Safely decode token with user key, check type and csrf value.
    let (_, csrf_key) = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::ResetPasswordToken,
        &key.value,
        token,
    )?;
    let csrf_key = csrf_key.ok_or_else(|| Error::BadRequest)?;
    csrf_check(driver, &csrf_key)?;

    // Update user password.
    core::user::update_password_by_id(driver, Some(service), &user.id, password)
}

/// User update email request.
pub fn update_email(
    driver: &Driver,
    service: &Service,
    key: Option<&str>,
    token: Option<&str>,
    password: &str,
    new_email: &str,
    revoke_token_expires: i64,
) -> Result<(User, String, String), Error> {
    // Get user and key using verified token or key, check is active and password match.
    let user_id = key_or_token_verify(driver, service, key, token)?;
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    let key = key_read_by_user(driver, service, &user)?;
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
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    Ok((user, old_email, revoke_token))
}

/// User update email revoke request.
pub fn update_email_revoke(
    driver: &Driver,
    service: &Service,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, do not check is active.
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

    // Disable user and all keys associated with user.
    core::user::update_by_id(driver, Some(service), &user.id, Some(false), None)?;
    let count =
        core::key::update_many_by_user_id(driver, Some(service), &user.id, Some(false), None)?;
    Ok(count + 1)
}

/// User update password request.
pub fn update_password(
    driver: &Driver,
    service: &Service,
    key: Option<&str>,
    token: Option<&str>,
    password: &str,
    new_password: &str,
    revoke_token_expires: i64,
) -> Result<(User, String), Error> {
    // Get user and key using verified token or key, check is active and password match.
    let user_id = key_or_token_verify(driver, service, key, token)?;
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    let key = key_read_by_user(driver, service, &user)?;
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
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    Ok((user, revoke_token))
}

/// User update password revoke request.
pub fn update_password_revoke(
    driver: &Driver,
    service: &Service,
    token: &str,
) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, do not check is active.
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

    // Disable user and all keys associated with user.
    core::user::update_by_id(driver, Some(service), &user.id, Some(false), None)?;
    let count =
        core::key::update_many_by_user_id(driver, Some(service), &user.id, Some(false), None)?;
    Ok(count + 1)
}

/// Verify user key.
pub fn key_verify(driver: &Driver, service: &Service, key: &str) -> Result<UserKey, Error> {
    // Get key, check is active and associated with user.
    let key = key_read_by_user_value(driver, service, key)?;
    let user_id = key.user_id.ok_or_else(|| Error::BadRequest)?;

    // Return key.
    Ok(UserKey {
        user_id,
        key: key.value,
    })
}

/// Revoke user key.
pub fn key_revoke(driver: &Driver, service: &Service, key: &str) -> Result<usize, Error> {
    // Get key, do not check is active.
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;

    // Disable key.
    core::key::update_by_id(driver, Some(service), &key.id, Some(false), None)?;
    Ok(1)
}

/// Verify user token.
pub fn token_verify(
    driver: &Driver,
    service: &Service,
    token: &str,
) -> Result<UserTokenPartial, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, check is active.
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    let key = key_read_by_user(driver, service, &user)?;

    // Safely decode token with user key, check type.
    let (access_token_expires, _) = core::jwt::decode_token(
        &service.id,
        &user.id,
        core::jwt::ClaimsType::AccessToken,
        &key.value,
        token,
    )?;

    // Return partial token.
    Ok(UserTokenPartial {
        user_id: user.id.to_owned(),
        access_token: token.to_owned(),
        access_token_expires: access_token_expires,
    })
}

/// Refresh token.
pub fn token_refresh(
    driver: &Driver,
    service: &Service,
    token: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<UserToken, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, _) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, check is active.
    let user = user_read_by_id(driver, Some(service), &user_id)?;
    let key = key_read_by_user(driver, service, &user)?;

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
pub fn token_revoke(driver: &Driver, service: &Service, token: &str) -> Result<usize, Error> {
    // Unsafely decode token to get user identifier, used to read key for safe token decode.
    let (user_id, token_type) = core::jwt::decode_unsafe(token, &service.id)?;

    // Get user and key, do not check is active.
    let user = core::user::read_by_id(driver, Some(service), &user_id)?
        .ok_or_else(|| Error::BadRequest)?;
    let key = core::key::read_by_user(driver, service, &user)?.ok_or_else(|| Error::BadRequest)?;

    // Safely decode token with user key, if it has CSRF key, invalidate it now.
    let (_, token_csrf) =
        core::jwt::decode_token(&service.id, &user.id, token_type, &key.value, token)?;
    if let Some(token_csrf) = token_csrf {
        core::csrf::read_by_key(driver, &token_csrf)?;
    }
    // Disable key associated with token.
    core::key::update_by_id(driver, Some(service), &key.id, Some(false), None)?;
    Ok(1)
}

/// OAuth2 user login.
pub fn oauth2_login(
    driver: &Driver,
    service_id: &str,
    email: &str,
    access_token_expires: i64,
    refresh_token_expires: i64,
) -> Result<(Service, UserToken), Error> {
    // Get service, user and key, check is active.
    let service = service_read_by_id(driver, service_id)?;
    let user = user_read_by_email(driver, Some(&service), email)?;
    let key = key_read_by_user(driver, &service, &user)?;

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
/// Also checks service is active, returns bad request if inactive.
fn service_read_by_id(driver: &Driver, service_id: &str) -> Result<Service, Error> {
    let service = driver
        .service_read_by_id(service_id)
        .map_err(Error::Driver)?
        .ok_or_else(|| Error::BadRequest)?;
    if !service.is_active {
        return Err(Error::BadRequest);
    }
    Ok(service)
}

/// Read user by ID.
/// Also checks user is active, returns bad request if inactive.
fn user_read_by_id(
    driver: &Driver,
    service_mask: Option<&Service>,
    id: &str,
) -> Result<User, Error> {
    let user =
        core::user::read_by_id(driver, service_mask, id)?.ok_or_else(|| Error::BadRequest)?;
    if !user.is_active {
        return Err(Error::BadRequest);
    }
    Ok(user)
}

/// Read user by email address.
/// Also checks user is active, returns bad request if inactive.
fn user_read_by_email(
    driver: &Driver,
    service_mask: Option<&Service>,
    email: &str,
) -> Result<User, Error> {
    let user =
        core::user::read_by_email(driver, service_mask, email)?.ok_or_else(|| Error::BadRequest)?;
    if !user.is_active {
        return Err(Error::BadRequest);
    }
    Ok(user)
}

/// Read key by user reference.
/// Also checks key is active, returns bad request if inactive.
fn key_read_by_user(driver: &Driver, service: &Service, user: &User) -> Result<Key, Error> {
    let key = core::key::read_by_user(driver, &service, &user)?.ok_or_else(|| Error::BadRequest)?;
    if !key.is_active {
        return Err(Error::BadRequest);
    }
    Ok(key)
}

/// Read key by user value.
/// Also checks key is active, returns bad request if inactive.
fn key_read_by_user_value(driver: &Driver, service: &Service, key: &str) -> Result<Key, Error> {
    let key =
        core::key::read_by_user_value(driver, service, key)?.ok_or_else(|| Error::BadRequest)?;
    if !key.is_active {
        return Err(Error::BadRequest);
    }
    Ok(key)
}

/// Get user ID from valid key or token.
fn key_or_token_verify(
    driver: &Driver,
    service: &Service,
    key: Option<&str>,
    token: Option<&str>,
) -> Result<String, Error> {
    match key {
        Some(key) => {
            let user_key = key_verify(driver, service, key)?;
            Ok(user_key.user_id)
        }
        None => match token {
            Some(token) => {
                let user_token = token_verify(driver, service, token)?;
                Ok(user_token.user_id)
            }
            None => Err(Error::Forbidden),
        },
    }
}

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

fn csrf_create(driver: &Driver, service: &Service, token_expires: i64) -> Result<Csrf, Error> {
    let csrf_key = uuid::Uuid::new_v4().to_simple().to_string();
    core::csrf::create(driver, service, &csrf_key, &csrf_key, token_expires)
}

fn csrf_check(driver: &Driver, csrf_key: &str) -> Result<(), Error> {
    core::csrf::read_by_key(driver, csrf_key)?
        .ok_or_else(|| Error::BadRequest)
        .map(|_| ())
}
