use crate::{
    AuditBuilder, AuditMessage, AuditMeta, AuditType, CoreError, CoreResult, Driver, Service, User,
};
use chrono::{DateTime, Utc};
use libreauth::key::KeyBuilder;
use std::fmt;
use uuid::Uuid;

// TODO(refactor): Use service_mask in functions to limit results, etc. Add tests for this.
// TODO(refactor): Use _audit unused, finish audit logs for routes, add optional properties.
// TODO(refactor): Improve key, user, service list query options (order by name, ...).
// TODO(refactor): Service callback URL per provider.
// TODO(refactor): User last login, key last use information.
// TODO(refactor): Respect allow_ key flags.

/// Key value size in bytes.
pub const KEY_VALUE_BYTES: usize = 21;

/// Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub allow_key: bool,
    pub allow_token: bool,
    pub allow_totp: bool,
    pub name: String,
    pub value: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Key {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tupdated_at {}", self.updated_at)?;
        write!(f, "\n\tis_enabled {}", self.is_enabled)?;
        write!(f, "\n\tis_revoked {}", self.is_revoked)?;
        write!(f, "\n\tallow_key {}", self.allow_key)?;
        write!(f, "\n\tallow_token {}", self.allow_token)?;
        write!(f, "\n\tallow_totp {}", self.allow_totp)?;
        write!(f, "\n\tname {}", self.name)?;
        write!(f, "\n\tvalue {}", self.value)?;
        if let Some(service_id) = &self.service_id {
            write!(f, "\n\tservice_id {}", service_id)?;
        }
        if let Some(user_id) = &self.user_id {
            write!(f, "\n\tuser_id {}", user_id)?;
        }
        Ok(())
    }
}

/// Key list.
#[derive(Debug)]
pub enum KeyList {
    Limit(i64),
    IdGt(Uuid, i64),
    IdLt(Uuid, i64),
}

/// Key count.
pub enum KeyCount {
    AllowToken(Uuid, Uuid),
    AllowTotp(Uuid, Uuid),
}

/// Key create data.
pub struct KeyCreate {
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub allow_key: bool,
    pub allow_token: bool,
    pub allow_totp: bool,
    pub name: String,
    pub value: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

/// Key read by service ID and user ID.
#[derive(Debug)]
pub struct KeyReadUserId {
    pub service_id: Uuid,
    pub user_id: Uuid,
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub allow_key: bool,
    pub allow_token: bool,
    pub allow_totp: bool,
}

/// Key read.
#[derive(Debug)]
pub enum KeyRead {
    Id(Uuid),
    UserId(KeyReadUserId),
    RootValue(String),
    ServiceValue(String),
    UserValue(Uuid, String),
}

/// Key update data.
pub struct KeyUpdate {
    pub is_enabled: Option<bool>,
    pub is_revoked: Option<bool>,
    pub allow_key: Option<bool>,
    pub allow_token: Option<bool>,
    pub allow_totp: Option<bool>,
    pub name: Option<String>,
}

impl Key {
    /// Authenticate root key.
    pub fn authenticate_root(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<AuditBuilder> {
        let mut audit = AuditBuilder::new(audit_meta);

        match key_value {
            Some(key_value) => Key::read_by_root_value(driver, &mut audit, key_value)
                .and_then(|key| match key.ok_or_else(|| CoreError::Forbidden) {
                    Ok(key) => {
                        audit.set_key(Some(&key));
                        Ok(key)
                    }
                    Err(err) => {
                        audit.create_internal(
                            driver,
                            AuditType::AuthenticateError,
                            AuditMessage::KeyNotFound,
                        );
                        Err(err)
                    }
                })
                .map(|_key| audit),
            None => {
                audit.create_internal(
                    driver,
                    AuditType::AuthenticateError,
                    AuditMessage::KeyUndefined,
                );
                Err(CoreError::Forbidden)
            }
        }
    }

    /// Authenticate service key.
    pub fn authenticate_service(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<(Service, AuditBuilder)> {
        let mut audit = AuditBuilder::new(audit_meta);

        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, &mut audit, key_value)
                .and_then(|key| match key.ok_or_else(|| CoreError::Forbidden) {
                    Ok(key) => {
                        audit.set_key(Some(&key));
                        Ok(key)
                    }
                    Err(err) => {
                        audit.create_internal(
                            driver,
                            AuditType::AuthenticateError,
                            AuditMessage::KeyNotFound,
                        );
                        Err(err)
                    }
                })
                .and_then(
                    |key| match key.service_id.ok_or_else(|| CoreError::Forbidden) {
                        Ok(service_id) => Ok(service_id),
                        Err(err) => {
                            audit.create_internal(
                                driver,
                                AuditType::AuthenticateError,
                                AuditMessage::KeyInvalid,
                            );
                            Err(err)
                        }
                    },
                )
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => {
                audit.create_internal(
                    driver,
                    AuditType::AuthenticateError,
                    AuditMessage::KeyUndefined,
                );
                Err(CoreError::Forbidden)
            }
        }
    }

    /// Authenticate service or root key.
    pub fn authenticate(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<(Option<Service>, AuditBuilder)> {
        let key_value_1 = key_value.to_owned();
        let audit_meta_copy = audit_meta.clone();

        Key::try_authenticate_service(driver, audit_meta, key_value)
            .map(|(service, audit)| (Some(service), audit))
            .or_else(move |err| match err {
                CoreError::Forbidden => {
                    Key::authenticate_root(driver, audit_meta_copy, key_value_1)
                        .map(|audit| (None, audit))
                }
                _ => Err(err),
            })
    }

    /// Authenticate service key, in case key does not exist or is not a service key, do not create audit log.
    /// This is used in cases where a key may be a service or root key, audit logs will be created by root key
    /// handler in case the key does not exist or is invalid.
    fn try_authenticate_service(
        driver: &dyn Driver,
        audit_meta: AuditMeta,
        key_value: Option<String>,
    ) -> CoreResult<(Service, AuditBuilder)> {
        let mut audit = AuditBuilder::new(audit_meta);

        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, &mut audit, key_value)
                .and_then(|key| key.ok_or_else(|| CoreError::Forbidden))
                .and_then(|key| key.service_id.ok_or_else(|| CoreError::Forbidden))
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => Err(CoreError::Forbidden),
        }
    }

    fn authenticate_service_inner(
        driver: &dyn Driver,
        mut audit: AuditBuilder,
        service_id: Uuid,
    ) -> CoreResult<(Service, AuditBuilder)> {
        Service::read_opt(driver, None, &mut audit, service_id).and_then(|service| {
            match service.ok_or_else(|| CoreError::Forbidden) {
                Ok(service) => {
                    audit.set_service(Some(&service));
                    Ok((service, audit))
                }
                Err(err) => {
                    audit.create_internal(
                        driver,
                        AuditType::AuthenticateError,
                        AuditMessage::ServiceNotFound,
                    );
                    Err(err)
                }
            }
        })
    }

    /// List keys using query.
    pub fn list(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        list: &KeyList,
    ) -> CoreResult<Vec<Key>> {
        let service_id_mask = service_mask.map(|s| &s.id);
        driver
            .key_list(list, service_id_mask)
            .map_err(CoreError::Driver)
    }

    /// Create root key.
    pub fn create_root(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        name: String,
    ) -> CoreResult<Key> {
        let value = Key::value_generate();
        let create = KeyCreate {
            is_enabled,
            is_revoked: false,
            allow_key: true,
            allow_token: false,
            allow_totp: false,
            name,
            value,
            service_id: None,
            user_id: None,
        };
        driver.key_create(&create).map_err(CoreError::Driver)
    }

    /// Create service key.
    pub fn create_service(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        name: String,
        service_id: Uuid,
    ) -> CoreResult<Key> {
        let value = Key::value_generate();
        let create = KeyCreate {
            is_enabled,
            is_revoked: false,
            allow_key: true,
            allow_token: false,
            allow_totp: false,
            name,
            value,
            service_id: Some(service_id),
            user_id: None,
        };
        driver.key_create(&create).map_err(CoreError::Driver)
    }

    /// Create user key.
    pub fn create_user(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        allow_key: bool,
        allow_token: bool,
        allow_totp: bool,
        name: String,
        service_id: Uuid,
        user_id: Uuid,
    ) -> CoreResult<Key> {
        if is_enabled {
            if allow_token {
                let count = KeyCount::AllowToken(service_id, user_id);
                let count = driver.key_count(&count)?;
                if count != 0 {
                    return Err(CoreError::BadRequest);
                }
            }
            if allow_totp {
                let count = KeyCount::AllowTotp(service_id, user_id);
                let count = driver.key_count(&count)?;
                if count != 0 {
                    return Err(CoreError::BadRequest);
                }
            }
        }

        let value = Key::value_generate();
        let create = KeyCreate {
            is_enabled,
            is_revoked: false,
            allow_key,
            allow_token,
            allow_totp,
            name,
            value,
            service_id: Some(service_id),
            user_id: Some(user_id),
        };
        driver.key_create(&create).map_err(CoreError::Driver)
    }

    /// Read key (optional).
    pub fn read_opt(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::Id(id);
        driver.key_read_opt(&read).map_err(CoreError::Driver)
    }

    /// Read key by user where key is enabled and not revoked.
    pub fn read_by_user(
        driver: &dyn Driver,
        service: &Service,
        _audit: &mut AuditBuilder,
        user: &User,
        allow_key: bool,
        allow_token: bool,
        allow_totp: bool,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::UserId(KeyReadUserId {
            service_id: service.id,
            user_id: user.id,
            is_enabled: true,
            is_revoked: false,
            allow_key,
            allow_token,
            allow_totp,
        });
        driver.key_read_opt(&read).map_err(CoreError::Driver)
    }

    /// Read key by value (root only).
    pub fn read_by_root_value(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        value: String,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::RootValue(value);
        driver.key_read_opt(&read).map_err(CoreError::Driver)
    }

    /// Read key by value (services only).
    pub fn read_by_service_value(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        value: String,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::ServiceValue(value);
        driver.key_read_opt(&read).map_err(CoreError::Driver)
    }

    /// Read key by value (users only).
    pub fn read_by_user_value(
        driver: &dyn Driver,
        service: &Service,
        _audit: &mut AuditBuilder,
        value: String,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::UserValue(service.id, value);
        driver.key_read_opt(&read).map_err(CoreError::Driver)
    }

    /// Update key.
    pub fn update(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        allow_key: Option<bool>,
        allow_token: Option<bool>,
        allow_totp: Option<bool>,
        name: Option<String>,
    ) -> CoreResult<Key> {
        let update = KeyUpdate {
            is_enabled,
            is_revoked,
            allow_key,
            allow_token,
            allow_totp,
            name,
        };
        driver.key_update(&id, &update).map_err(CoreError::Driver)
    }

    /// Update many keys by user ID.
    pub fn update_many(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        user_id: Uuid,
        is_enabled: Option<bool>,
        is_revoked: Option<bool>,
        name: Option<String>,
    ) -> CoreResult<usize> {
        let update = KeyUpdate {
            is_enabled,
            is_revoked,
            allow_key: None,
            allow_token: None,
            allow_totp: None,
            name,
        };
        driver
            .key_update_many(&user_id, &update)
            .map_err(CoreError::Driver)
    }

    /// Delete key.
    pub fn delete(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<usize> {
        driver.key_delete(&id).map_err(CoreError::Driver)
    }

    /// Create new key value from random bytes.
    pub fn value_generate() -> String {
        KeyBuilder::new()
            .size(KEY_VALUE_BYTES)
            .generate()
            .as_base32()
    }
}
