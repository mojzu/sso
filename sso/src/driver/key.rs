use crate::{impl_enum_to_from_string, AuditDiff, AuditDiffBuilder, AuditSubject};
use chrono::{DateTime, Utc};
use libreauth::key::KeyBuilder;
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

// TODO(refactor): Add name ge/le key, service list query options.

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
    pub service_id_mask: Option<Uuid>,
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

impl KeyCreate {
    /// Create root key.
    pub fn root<N>(is_enabled: bool, name: N) -> Self
    where
        N: Into<String>,
    {
        let value = value_generate();
        Self {
            is_enabled,
            is_revoked: false,
            type_: KeyType::Key,
            name: name.into(),
            value,
            service_id: None,
            user_id: None,
        }
    }

    /// Create service key.
    pub fn service<N>(is_enabled: bool, name: N, service_id: Uuid) -> Self
    where
        N: Into<String>,
    {
        let value = value_generate();
        Self {
            is_enabled,
            is_revoked: false,
            type_: KeyType::Key,
            name: name.into(),
            value,
            service_id: Some(service_id),
            user_id: None,
        }
    }

    /// Create user key.
    pub fn user<N>(
        is_enabled: bool,
        type_: KeyType,
        name: N,
        service_id: Uuid,
        user_id: Uuid,
    ) -> Self
    where
        N: Into<String>,
    {
        let value = value_generate();
        Self {
            is_enabled,
            is_revoked: false,
            type_,
            name: name.into(),
            value,
            service_id: Some(service_id),
            user_id: Some(user_id),
        }
    }
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
    Id(Uuid, Option<Uuid>),
    RootValue(String),
    ServiceValue(String),
    UserId(KeyReadUserId),
    UserValue(KeyReadUserValue),
}

impl KeyRead {
    pub fn user_id(
        service_id: Uuid,
        user_id: Uuid,
        is_enabled: bool,
        is_revoked: bool,
        type_: KeyType,
    ) -> Self {
        Self::UserId(KeyReadUserId {
            service_id,
            user_id,
            is_enabled,
            is_revoked,
            type_,
        })
    }

    pub fn user_value<V>(
        service_id: Uuid,
        value: V,
        is_enabled: bool,
        is_revoked: bool,
        type_: KeyType,
    ) -> Self
    where
        V: Into<String>,
    {
        Self::UserValue(KeyReadUserValue {
            service_id,
            value: value.into(),
            is_enabled,
            is_revoked,
            type_,
        })
    }
}

/// Key update data.
#[derive(Debug)]
pub struct KeyUpdate {
    pub is_enabled: Option<bool>,
    pub is_revoked: Option<bool>,
    pub name: Option<String>,
}

/// Generate new key value from random bytes.
fn value_generate() -> String {
    KeyBuilder::new()
        .size(KEY_VALUE_BYTES)
        .generate()
        .as_base32()
}
