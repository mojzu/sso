use crate::{
    impl_enum_to_from_string, AuditBuilder, AuditMessage, AuditMeta, AuditType, CoreError,
    CoreResult, Driver, Service, User, UserRead,
};
use chrono::{DateTime, Utc};
use libreauth::key::KeyBuilder;
use std::fmt;
use uuid::Uuid;

// TODO(refactor): Use service_mask in functions to limit results, etc. Add tests for this.
// TODO(refactor): Use _audit unused, finish audit logs for routes, add optional properties.
// TODO(refactor): Improve key, user, service list query options (order by name, text search, ...).
// TODO(refactor): User last login, key last use information (calculate in SQL).
// TODO(refactor): Audit key value reads, separate endpoint?

/// Key value size in bytes.
pub const KEY_VALUE_BYTES: usize = 21;

/// Key types.
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub enum KeyType {
    Key,
    Token,
    Totp,
}

impl_enum_to_from_string!(KeyType);

/// Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Key {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub type_: KeyType,
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
        write!(f, "\n\ttype {}", self.type_.to_string().unwrap())?;
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

/// Key list query.
#[derive(Debug)]
pub enum KeyListQuery {
    Limit(i64),
    IdGt(Uuid, i64),
    IdLt(Uuid, i64),
}

/// Key list filter.
#[derive(Debug)]
pub struct KeyListFilter {
    pub id: Option<Vec<Uuid>>,
    pub is_enabled: Option<bool>,
    pub is_revoked: Option<bool>,
    pub type_: Option<Vec<KeyType>>,
    pub service_id: Option<Vec<Uuid>>,
    pub user_id: Option<Vec<Uuid>>,
}

/// Key list.
#[derive(Debug)]
pub struct KeyList<'a> {
    pub query: &'a KeyListQuery,
    pub filter: &'a KeyListFilter,
    pub service_id_mask: Option<&'a Uuid>,
}

/// Key count.
#[derive(Debug)]
pub enum KeyCount {
    Token(Uuid, Uuid),
    Totp(Uuid, Uuid),
}

/// Key create data.
#[derive(Debug)]
pub struct KeyCreate {
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub type_: KeyType,
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
    pub type_: KeyType,
}

/// Key read by service ID and user value.
#[derive(Debug)]
pub struct KeyReadUserValue {
    pub service_id: Uuid,
    pub value: String,
    pub is_enabled: bool,
    pub is_revoked: bool,
    pub type_: KeyType,
}

/// Key read.
#[derive(Debug)]
pub enum KeyRead {
    Id(Uuid),
    RootValue(String),
    ServiceValue(String),
    UserId(KeyReadUserId),
    UserValue(KeyReadUserValue),
}

/// Key update data.
#[derive(Debug)]
pub struct KeyUpdate {
    pub is_enabled: Option<bool>,
    pub is_revoked: Option<bool>,
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
                .and_then(|key| match key.ok_or_else(|| CoreError::Unauthorised) {
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
                Err(CoreError::Unauthorised)
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
                .and_then(|key| match key.ok_or_else(|| CoreError::Unauthorised) {
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
                    |key| match key.service_id.ok_or_else(|| CoreError::Unauthorised) {
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
                Err(CoreError::Unauthorised)
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
                CoreError::Unauthorised => {
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
                .and_then(|key| key.ok_or_else(|| CoreError::Unauthorised))
                .and_then(|key| key.service_id.ok_or_else(|| CoreError::Unauthorised))
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => Err(CoreError::Unauthorised),
        }
    }

    fn authenticate_service_inner(
        driver: &dyn Driver,
        mut audit: AuditBuilder,
        service_id: Uuid,
    ) -> CoreResult<(Service, AuditBuilder)> {
        Service::read_opt(driver, None, &mut audit, &service_id).and_then(|service| {
            match service.ok_or_else(|| CoreError::Unauthorised) {
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
        query: &KeyListQuery,
        filter: &KeyListFilter,
    ) -> CoreResult<Vec<Key>> {
        let service_id_mask = service_mask.map(|s| &s.id);
        let list = KeyList {
            query,
            filter,
            service_id_mask,
        };
        driver.key_list(&list).map_err(CoreError::Driver)
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
            type_: KeyType::Key,
            name,
            value,
            service_id: None,
            user_id: None,
        };
        driver.key_create(&create).map_err(CoreError::Driver)
    }

    /// Create service key.
    /// Returns bad request if service does not exist.
    pub fn create_service(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        is_enabled: bool,
        name: String,
        service_id: &Uuid,
    ) -> CoreResult<Key> {
        let service = Service::read_opt(driver, None, audit, service_id)?
            .ok_or_else(|| CoreError::BadRequest)?;

        let value = Key::value_generate();
        let create = KeyCreate {
            is_enabled,
            is_revoked: false,
            type_: KeyType::Key,
            name,
            value,
            service_id: Some(service.id),
            user_id: None,
        };
        driver.key_create(&create).map_err(CoreError::Driver)
    }

    /// Create user key.
    /// Returns bad request if more than one `Token` or `Totp` type would be enabled.
    /// Returns bad request if service or user does not exist.
    pub fn create_user(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        is_enabled: bool,
        type_: KeyType,
        name: String,
        service_id: &Uuid,
        user_id: &Uuid,
    ) -> CoreResult<Key> {
        if is_enabled {
            if type_ == KeyType::Token {
                let count = KeyCount::Token(*service_id, *user_id);
                let count = driver.key_count(&count)?;
                if count != 0 {
                    return Err(CoreError::BadRequest);
                }
            }
            if type_ == KeyType::Totp {
                let count = KeyCount::Totp(*service_id, *user_id);
                let count = driver.key_count(&count)?;
                if count != 0 {
                    return Err(CoreError::BadRequest);
                }
            }
        }
        let service = Service::read_opt(driver, None, audit, service_id)?
            .ok_or_else(|| CoreError::BadRequest)?;
        let user_read = UserRead::Id(*user_id);
        let user = User::read_opt(driver, None, audit, &user_read)?
            .ok_or_else(|| CoreError::BadRequest)?;

        let value = Key::value_generate();
        let create = KeyCreate {
            is_enabled,
            is_revoked: false,
            type_,
            name,
            value,
            service_id: Some(service.id),
            user_id: Some(user.id),
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

    /// Read key by user where key is enabled and not revoked.
    pub fn read_by_user(
        driver: &dyn Driver,
        service: &Service,
        _audit: &mut AuditBuilder,
        user: &User,
        type_: KeyType,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::UserId(KeyReadUserId {
            service_id: service.id,
            user_id: user.id,
            is_enabled: true,
            is_revoked: false,
            type_,
        });
        driver.key_read_opt(&read).map_err(CoreError::Driver)
    }

    /// Read key by value and type where key is enabled and not revoked.
    pub fn read_by_user_value(
        driver: &dyn Driver,
        service: &Service,
        _audit: &mut AuditBuilder,
        value: String,
        type_: KeyType,
    ) -> CoreResult<Option<Key>> {
        let read = KeyRead::UserValue(KeyReadUserValue {
            service_id: service.id,
            value,
            is_enabled: true,
            is_revoked: false,
            type_,
        });
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
        name: Option<String>,
    ) -> CoreResult<Key> {
        let update = KeyUpdate {
            is_enabled,
            is_revoked,
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
