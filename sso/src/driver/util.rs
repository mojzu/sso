use crate::{
    AuditBuilder, Driver, DriverError, DriverResult, KeyRead, KeyType, KeyWithValue, Service,
    ServiceRead, User, UserRead,
};
use libreauth::oath::TOTPBuilder;
use uuid::Uuid;

// TODO(refactor): Improve usability, composability of utility functions.
// Should be reusable units of code commonly called into from api module.

/// Verify TOTP code using key.
pub fn totp_verify(key: &str, code: &str) -> DriverResult<()> {
    let totp = TOTPBuilder::new()
        .base32_key(key)
        .finalize()
        .map_err(DriverError::libreauth_oath)?;

    if !totp.is_valid(&code) {
        Err(DriverError::TotpInvalid)
    } else {
        Ok(())
    }
}

/// Authenticate root key.
pub fn key_root_authenticate(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    key_value: Option<String>,
) -> DriverResult<()> {
    match key_value {
        Some(key_value) => {
            let read = KeyRead::RootValue(key_value);
            driver
                .key_read(&read)?
                .ok_or_else(|| DriverError::KeyNotFound)
                .map(|key| {
                    audit.key(Some(&key));
                    key
                })
                .map(|_key| ())
        }
        None => Err(DriverError::KeyUndefined),
    }
}

/// Authenticate service key.
pub fn key_service_authenticate(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    key_value: Option<String>,
) -> DriverResult<Service> {
    key_service_authenticate_try(driver, audit, key_value)
}

/// Authenticate service or root key.
pub fn key_authenticate(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    key_value: Option<String>,
) -> DriverResult<Option<Service>> {
    let key_value_1 = key_value.to_owned();

    key_service_authenticate_try(driver, audit, key_value)
        .map(Some)
        .or_else(move |_err| key_root_authenticate(driver, audit, key_value_1).map(|_| None))
}

fn key_service_authenticate_try(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    key_value: Option<String>,
) -> DriverResult<Service> {
    match key_value {
        Some(key_value) => driver
            .key_read(&KeyRead::ServiceValue(key_value))?
            .ok_or_else(|| DriverError::KeyNotFound)
            .and_then(|key| {
                key.service_id
                    .ok_or_else(|| DriverError::KeyServiceUndefined)
            })
            .and_then(|service_id| key_service_authenticate_inner(driver, audit, service_id)),
        None => Err(DriverError::KeyUndefined),
    }
}

fn key_service_authenticate_inner(
    driver: &dyn Driver,
    audit: &mut AuditBuilder,
    service_id: Uuid,
) -> DriverResult<Service> {
    let service = driver
        .service_read(&ServiceRead::new(service_id))?
        .ok_or_else(|| DriverError::ServiceNotFound)?
        .check()?;
    audit.service(Some(&service));
    Ok(service)
}

/// Read user by ID.
/// Checks user is enabled, returns bad request if disabled.
pub fn user_read_id_checked(
    driver: &dyn Driver,
    _service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    id: Uuid,
) -> DriverResult<User> {
    let read = UserRead::Id(id);
    let user = driver
        .user_read(&read)?
        .ok_or_else(|| DriverError::UserNotFound)?;
    audit.user(Some(&user));
    if !user.is_enabled {
        return Err(DriverError::UserDisabled);
    }
    Ok(user)
}

/// Unchecked read user by ID.
/// Does not check user is enabled.
pub fn user_read_id_unchecked(
    driver: &dyn Driver,
    _service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    id: Uuid,
) -> DriverResult<User> {
    let read = UserRead::Id(id);
    let user = driver
        .user_read(&read)?
        .ok_or_else(|| DriverError::UserNotFound)?;
    audit.user(Some(&user));
    Ok(user)
}

/// Read user by email address.
/// Also checks user is enabled, returns bad request if disabled.
pub fn user_read_email_checked(
    driver: &dyn Driver,
    _service_mask: Option<&Service>,
    audit: &mut AuditBuilder,
    email: String,
) -> DriverResult<User> {
    let read = UserRead::Email(email);
    let user = driver
        .user_read(&read)?
        .ok_or_else(|| DriverError::UserNotFound)?;
    audit.user(Some(&user));
    if !user.is_enabled {
        return Err(DriverError::UserDisabled);
    }
    Ok(user)
}

/// Read key by user reference and key type.
/// Also checks key is enabled and not revoked, returns bad request if disabled.
pub fn key_read_user_checked(
    driver: &dyn Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    user: &User,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(&KeyRead::user_id(
            service.id, user.id, true, false, key_type,
        ))?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    if !key.is_enabled {
        Err(DriverError::KeyDisabled)
    } else if key.is_revoked {
        Err(DriverError::KeyRevoked)
    } else {
        Ok(key)
    }
}

/// Unchecked read key by user reference.
/// Does not check key is enabled or not revoked.
pub fn key_read_user_unchecked(
    driver: &dyn Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    user: &User,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(&KeyRead::user_id(
            service.id, user.id, true, false, key_type,
        ))?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    Ok(key)
}

/// Read key by user value.
/// Also checks key is enabled and not revoked, returns bad request if disabled.
pub fn key_read_user_value_checked(
    driver: &dyn Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    key: String,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(&KeyRead::user_value(service.id, key, true, false, key_type))?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    if !key.is_enabled {
        Err(DriverError::KeyDisabled)
    } else if key.is_revoked {
        Err(DriverError::KeyRevoked)
    } else {
        Ok(key)
    }
}

/// Unchecked read key by user value.
/// Does not check key is enabled and not revoked.
pub fn key_read_user_value_unchecked(
    driver: &dyn Driver,
    service: &Service,
    audit: &mut AuditBuilder,
    key: String,
    key_type: KeyType,
) -> DriverResult<KeyWithValue> {
    let key = driver
        .key_read(&KeyRead::user_value(service.id, key, true, false, key_type))?
        .ok_or_else(|| DriverError::KeyNotFound)?;
    audit.user_key(Some(&key));
    Ok(key)
}
