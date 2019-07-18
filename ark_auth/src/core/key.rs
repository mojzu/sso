use crate::core::audit::{AuditBuilder, AuditMessage, AuditPath};
use crate::core::service::read_by_id as service_read_by_id;
use crate::core::{AuditMeta, Error, Key, KeyQuery, Service, User, DEFAULT_LIMIT};
use crate::driver;

// TODO(refactor): Use service_mask in functions to limit results, etc. Add tests for this.
// TODO(refactor): Use _audit unused, finish audit logs for routes, add optional properties.
// TODO(refactor): Improve key, user, service list query options (order by name, ...).

/// Authenticate root key.
pub fn authenticate_root(
    driver: &driver::Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
) -> Result<AuditBuilder, Error> {
    let mut audit = AuditBuilder::new(audit_meta);

    match key_value {
        Some(key_value) => read_by_root_value(driver, &mut audit, &key_value)
            .and_then(|key| match key.ok_or_else(|| Error::Forbidden) {
                Ok(key) => {
                    audit.set_key(Some(&key));
                    Ok(key)
                }
                Err(err) => {
                    audit.create_internal(
                        driver,
                        AuditPath::AuthenticationError,
                        AuditMessage::KeyNotFound,
                    );
                    Err(err)
                }
            })
            .map(|_key| audit),
        None => Err(Error::Forbidden),
    }
}

/// Authenticate service key.
pub fn authenticate_service(
    driver: &driver::Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
) -> Result<(Service, AuditBuilder), Error> {
    let mut audit = AuditBuilder::new(audit_meta);

    match key_value {
        Some(key_value) => read_by_service_value(driver, &mut audit, &key_value)
            .and_then(|key| match key.ok_or_else(|| Error::Forbidden) {
                Ok(key) => {
                    audit.set_key(Some(&key));
                    Ok(key)
                }
                Err(err) => {
                    audit.create_internal(
                        driver,
                        AuditPath::AuthenticationError,
                        AuditMessage::KeyNotFound,
                    );
                    Err(err)
                }
            })
            .and_then(|key| match key.service_id.ok_or_else(|| Error::Forbidden) {
                Ok(service_id) => Ok(service_id),
                Err(err) => {
                    audit.create_internal(
                        driver,
                        AuditPath::AuthenticationError,
                        AuditMessage::KeyInvalid,
                    );
                    Err(err)
                }
            })
            .and_then(|service_id| service_read_by_id(driver, None, &mut audit, &service_id))
            .and_then(|service| match service.ok_or_else(|| Error::Forbidden) {
                Ok(service) => {
                    audit.set_service(Some(&service));
                    Ok((service, audit))
                }
                Err(err) => {
                    audit.create_internal(
                        driver,
                        AuditPath::AuthenticationError,
                        AuditMessage::ServiceNotFound,
                    );
                    Err(err)
                }
            }),
        None => Err(Error::Forbidden),
    }
}

/// Authenticate service or root key.
pub fn authenticate(
    driver: &driver::Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
) -> Result<(Option<Service>, AuditBuilder), Error> {
    let key_value_1 = key_value.to_owned();
    let audit_meta_copy = audit_meta.clone();

    authenticate_service(driver, audit_meta, key_value)
        .map(|(service, audit)| (Some(service), audit))
        .or_else(move |err| match err {
            Error::Forbidden => {
                authenticate_root(driver, audit_meta_copy, key_value_1).map(|audit| (None, audit))
            }
            _ => Err(err),
        })
}

/// List keys using query.
pub fn list(
    driver: &driver::Driver,
    service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    query: &KeyQuery,
) -> Result<Vec<String>, Error> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);
    let service_mask = service_mask.map(|s| s.id.as_ref());

    match &query.gt {
        Some(gt) => driver
            .key_list_where_id_gt(gt, limit, service_mask)
            .map_err(Error::Driver),
        None => match &query.lt {
            Some(lt) => driver
                .key_list_where_id_lt(lt, limit, service_mask)
                .map_err(Error::Driver),
            None => driver
                .key_list_where_id_gt("", limit, service_mask)
                .map_err(Error::Driver),
        },
    }
}

/// Create root key.
pub fn create_root(
    driver: &driver::Driver,
    _audit: &mut AuditBuilder,
    is_enabled: bool,
    name: &str,
) -> Result<Key, Error> {
    let value = uuid::Uuid::new_v4().to_simple().to_string();
    driver
        .key_create(is_enabled, false, name, &value, None, None)
        .map_err(Error::Driver)
}

/// Create service key.
pub fn create_service(
    driver: &driver::Driver,
    _audit: &mut AuditBuilder,
    is_enabled: bool,
    name: &str,
    service_id: &str,
) -> Result<Key, Error> {
    let value = uuid::Uuid::new_v4().to_simple().to_string();
    driver
        .key_create(is_enabled, false, name, &value, Some(service_id), None)
        .map_err(Error::Driver)
}

/// Create user key.
pub fn create_user(
    driver: &driver::Driver,
    _audit: &mut AuditBuilder,
    is_enabled: bool,
    name: &str,
    service_id: &str,
    user_id: &str,
) -> Result<Key, Error> {
    let value = uuid::Uuid::new_v4().to_simple().to_string();
    driver
        .key_create(
            is_enabled,
            false,
            name,
            &value,
            Some(service_id),
            Some(user_id),
        )
        .map_err(Error::Driver)
}

/// Read key by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
) -> Result<Option<Key>, Error> {
    driver.key_read_by_id(id).map_err(Error::Driver)
}

/// Read key by user.
pub fn read_by_user(
    driver: &driver::Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    user: &User,
) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_user_id(&service.id, &user.id)
        .map_err(Error::Driver)
}

/// Read key by value (root only).
pub fn read_by_root_value(
    driver: &driver::Driver,
    _audit: &mut AuditBuilder,
    value: &str,
) -> Result<Option<Key>, Error> {
    driver.key_read_by_root_value(value).map_err(Error::Driver)
}

/// Read key by value (services only).
pub fn read_by_service_value(
    driver: &driver::Driver,
    _audit: &mut AuditBuilder,
    value: &str,
) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_service_value(value)
        .map_err(Error::Driver)
}

/// Read key by value (users only).
pub fn read_by_user_value(
    driver: &driver::Driver,
    service: &Service,
    _audit: &mut AuditBuilder,
    value: &str,
) -> Result<Option<Key>, Error> {
    driver
        .key_read_by_user_value(&service.id, value)
        .map_err(Error::Driver)
}

/// Update key by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
    is_enabled: Option<bool>,
    is_revoked: Option<bool>,
    name: Option<&str>,
) -> Result<Key, Error> {
    driver
        .key_update_by_id(id, is_enabled, is_revoked, name)
        .map_err(Error::Driver)
}

/// Update many keys by user ID.
pub fn update_many_by_user_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    user_id: &str,
    is_enabled: Option<bool>,
    is_revoked: Option<bool>,
    name: Option<&str>,
) -> Result<usize, Error> {
    driver
        .key_update_many_by_user_id(user_id, is_enabled, is_revoked, name)
        .map_err(Error::Driver)
}

/// Delete key by ID.
pub fn delete_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: &str,
) -> Result<usize, Error> {
    driver.key_delete_by_id(id).map_err(Error::Driver)
}

/// Delete all root keys.
pub fn delete_root(driver: &driver::Driver, _audit: &mut AuditBuilder) -> Result<usize, Error> {
    driver.key_delete_root().map_err(Error::Driver)
}
