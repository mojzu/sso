use crate::{
    impl_enum_to_from_string, AuditBuilder, AuditDiff, AuditDiffBuilder, AuditSubject, CoreCause,
    CoreError, CoreResult, Driver, Service, User, UserRead,
};
use chrono::{DateTime, Utc};
use libreauth::key::KeyBuilder;
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

// TODO(refactor): Use service_mask in functions to limit results, etc. Add tests for this.
// TODO(refactor): Improve key, user, service list query options (order by name, text search, ...).
// TODO(refactor): User last login, key last use information (calculate in SQL).
// TODO(refactor): Check audit logging in auth module, add tests.

/// Key value size in bytes.
pub const KEY_VALUE_BYTES: usize = 21;

/// Key types.
#[derive(Debug, Copy, PartialEq, Clone, Serialize, Deserialize)]
pub enum KeyType {
    Key,
    Token,
    Totp,
}

impl_enum_to_from_string!(KeyType, "");

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
        if let Some(service_id) = &self.service_id {
            write!(f, "\n\tservice_id {}", service_id)?;
        }
        if let Some(user_id) = &self.user_id {
            write!(f, "\n\tuser_id {}", user_id)?;
        }
        Ok(())
    }
}

impl AuditSubject for Key {
    fn subject(&self) -> String {
        format!("{}", self.id)
    }
}

impl AuditDiff for Key {
    fn diff(&self, previous: &Self) -> Value {
        AuditDiffBuilder::default()
            .compare("is_enabled", &self.is_enabled, &previous.is_enabled)
            .compare("is_revoked", &self.is_revoked, &previous.is_revoked)
            .compare("name", &self.name, &previous.name)
            .into_value()
    }
}

/// Key with value.
///
/// This is split from `Key` to make value private except when created
/// or read internally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyWithValue {
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

impl fmt::Display for KeyWithValue {
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

impl AuditSubject for KeyWithValue {
    fn subject(&self) -> String {
        format!("{}", self.id)
    }
}

impl From<KeyWithValue> for Key {
    fn from(k: KeyWithValue) -> Self {
        Self {
            created_at: k.created_at,
            updated_at: k.updated_at,
            id: k.id,
            is_enabled: k.is_enabled,
            is_revoked: k.is_revoked,
            type_: k.type_,
            name: k.name,
            service_id: k.service_id,
            user_id: k.user_id,
        }
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
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<()> {
        match key_value {
            Some(key_value) => Key::read_by_root_value(driver, key_value)
                .map(|key| {
                    audit.key(Some(&key));
                    key
                })
                .map(|_key| ()),
            None => Err(CoreError::Unauthorised(CoreCause::KeyUndefined)),
        }
        .map_err(Self::map_err_unauthorised)
    }

    /// Authenticate service key.
    pub fn authenticate_service(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<Service> {
        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, key_value)
                .and_then(|key| {
                    audit.key(Some(&key));

                    match key
                        .service_id
                        .ok_or_else(|| CoreError::Unauthorised(CoreCause::KeyInvalid))
                    {
                        Ok(service_id) => Ok(service_id),
                        Err(err) => Err(err),
                    }
                })
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => Err(CoreError::Unauthorised(CoreCause::KeyUndefined)),
        }
        .map_err(Self::map_err_unauthorised)
    }

    /// Authenticate service or root key.
    pub fn authenticate(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<Option<Service>> {
        let key_value_1 = key_value.to_owned();

        Key::try_authenticate_service(driver, audit, key_value)
            .map(Some)
            .or_else(move |err| match err {
                CoreError::Unauthorised(_) => {
                    Key::authenticate_root(driver, audit, key_value_1).map(|_| None)
                }
                _ => Err(err),
            })
            .map_err(Self::map_err_unauthorised)
    }

    /// Authenticate service key, in case key does not exist or is not a service key, do not create audit log.
    /// This is used in cases where a key may be a service or root key, audit logs will be created by root key
    /// handler in case the key does not exist or is invalid.
    fn try_authenticate_service(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
    ) -> CoreResult<Service> {
        match key_value {
            Some(key_value) => Key::read_by_service_value(driver, key_value)
                .and_then(|key| {
                    key.service_id
                        .ok_or_else(|| CoreError::Unauthorised(CoreCause::KeyInvalid))
                })
                .and_then(|service_id| Key::authenticate_service_inner(driver, audit, service_id)),
            None => Err(CoreError::Unauthorised(CoreCause::KeyUndefined)),
        }
        .map_err(Self::map_err_unauthorised)
    }

    fn authenticate_service_inner(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        service_id: Uuid,
    ) -> CoreResult<Service> {
        let service = Service::read(driver, None, &service_id)?;
        audit.service(Some(&service));
        Ok(service)
    }

    fn map_err_unauthorised(e: CoreError) -> CoreError {
        match e {
            CoreError::BadRequest(cause) => CoreError::Unauthorised(cause),
            CoreError::Forbidden(cause) => CoreError::Unauthorised(cause),
            CoreError::NotFound(cause) => CoreError::Unauthorised(cause),
            _ => e,
        }
    }

    /// List keys using query.
    pub fn list(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
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
        is_enabled: bool,
        name: String,
    ) -> CoreResult<KeyWithValue> {
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
        is_enabled: bool,
        name: String,
        service_id: &Uuid,
    ) -> CoreResult<KeyWithValue> {
        let service = Service::read_opt(driver, None, service_id)?
            .ok_or_else(|| CoreError::BadRequest(CoreCause::ServiceNotFound))?;

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
        is_enabled: bool,
        type_: KeyType,
        name: String,
        service_id: &Uuid,
        user_id: &Uuid,
    ) -> CoreResult<KeyWithValue> {
        if is_enabled {
            if type_ == KeyType::Token {
                let count = KeyCount::Token(*service_id, *user_id);
                let count = driver.key_count(&count)?;
                if count != 0 {
                    return Err(CoreError::BadRequest(CoreCause::UserKeyTooManyEnabledToken));
                }
            }
            if type_ == KeyType::Totp {
                let count = KeyCount::Totp(*service_id, *user_id);
                let count = driver.key_count(&count)?;
                if count != 0 {
                    return Err(CoreError::BadRequest(CoreCause::UserKeyTooManyEnabledTotp));
                }
            }
        }
        let service = Service::read_opt(driver, None, service_id)?
            .ok_or_else(|| CoreError::BadRequest(CoreCause::ServiceNotFound))?;
        let user_read = UserRead::Id(*user_id);
        let user = User::read_opt(driver, None, &user_read)?
            .ok_or_else(|| CoreError::BadRequest(CoreCause::UserNotFound))?;

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

    /// Read key.
    pub fn read(
        driver: &dyn Driver,
        service_mask: Option<&Service>,
        read: &KeyRead,
    ) -> CoreResult<KeyWithValue> {
        Self::read_opt(driver, service_mask, read)?
            .ok_or_else(|| CoreError::NotFound(CoreCause::KeyNotFound))
    }

    /// Read key (optional).
    pub fn read_opt(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        read: &KeyRead,
    ) -> CoreResult<Option<KeyWithValue>> {
        driver.key_read_opt(read).map_err(CoreError::Driver)
    }

    /// Read key by value (root only).
    pub fn read_by_root_value(driver: &dyn Driver, value: String) -> CoreResult<KeyWithValue> {
        let read = KeyRead::RootValue(value);
        Self::read(driver, None, &read)
    }

    /// Read key by value (services only).
    pub fn read_by_service_value(driver: &dyn Driver, value: String) -> CoreResult<KeyWithValue> {
        let read = KeyRead::ServiceValue(value);
        Self::read(driver, None, &read)
    }

    /// Read key by user where key is enabled and not revoked.
    pub fn read_by_user(
        driver: &dyn Driver,
        service: &Service,
        user: &User,
        type_: KeyType,
    ) -> CoreResult<KeyWithValue> {
        let read = KeyRead::UserId(KeyReadUserId {
            service_id: service.id,
            user_id: user.id,
            is_enabled: true,
            is_revoked: false,
            type_,
        });
        Self::read(driver, Some(service), &read)
    }

    /// Read key by value and type where key is enabled and not revoked.
    pub fn read_by_user_value(
        driver: &dyn Driver,
        service: &Service,
        value: String,
        type_: KeyType,
    ) -> CoreResult<KeyWithValue> {
        let read = KeyRead::UserValue(KeyReadUserValue {
            service_id: service.id,
            value,
            is_enabled: true,
            is_revoked: false,
            type_,
        });
        Self::read(driver, Some(service), &read)
    }

    /// Update key.
    pub fn update(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
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
        id: Uuid,
    ) -> CoreResult<()> {
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
