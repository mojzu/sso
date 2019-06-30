use crate::core::{Audit, AuditQuery, Key, KeyQuery, Service, ServiceQuery, User, UserQuery};
use crate::server::{validate, FromJsonValue};
use serde_json::Value;
use validator::Validate;

// Audit types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditListQuery {
    #[validate(custom = "validate::id")]
    pub gt: Option<String>,
    #[validate(custom = "validate::id")]
    pub lt: Option<String>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<i64>,
}

impl FromJsonValue<AuditListQuery> for AuditListQuery {}

impl From<AuditListQuery> for AuditQuery {
    fn from(query: AuditListQuery) -> AuditQuery {
        AuditQuery {
            gt: query.gt,
            lt: query.lt,
            limit: query.limit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditListResponse {
    pub meta: AuditQuery,
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditCreateBody {
    #[validate(custom = "validate::path")]
    pub path: String,
    pub data: Value,
    #[validate(custom = "validate::id")]
    pub user_id: Option<String>,
    #[validate(custom = "validate::id")]
    pub user_key_id: Option<String>,
}

impl FromJsonValue<AuditCreateBody> for AuditCreateBody {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditCreateResponse {
    pub data: Audit,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditReadResponse {
    pub data: Audit,
}

// Authentication types.

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthPasswordMeta {
    pub password_strength: Option<u8>,
    pub password_pwned: Option<bool>,
}

impl Default for AuthPasswordMeta {
    fn default() -> Self {
        AuthPasswordMeta {
            password_strength: None,
            password_pwned: None,
        }
    }
}

// Key types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyListQuery {
    #[validate(custom = "validate::id")]
    pub gt: Option<String>,
    #[validate(custom = "validate::id")]
    pub lt: Option<String>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<i64>,
}

impl FromJsonValue<KeyListQuery> for KeyListQuery {}

impl From<KeyListQuery> for KeyQuery {
    fn from(query: KeyListQuery) -> KeyQuery {
        KeyQuery {
            gt: query.gt,
            lt: query.lt,
            limit: query.limit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyListResponse {
    pub meta: KeyQuery,
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyCreateBody {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(custom = "validate::id")]
    pub service_id: Option<String>,
    #[validate(custom = "validate::id")]
    pub user_id: Option<String>,
}

impl FromJsonValue<KeyCreateBody> for KeyCreateBody {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyReadResponse {
    pub data: Key,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyUpdateBody {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
}

impl FromJsonValue<KeyUpdateBody> for KeyUpdateBody {}

// Service types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceListQuery {
    #[validate(custom = "validate::id")]
    pub gt: Option<String>,
    #[validate(custom = "validate::id")]
    pub lt: Option<String>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<i64>,
}

impl From<ServiceListQuery> for ServiceQuery {
    fn from(query: ServiceListQuery) -> ServiceQuery {
        ServiceQuery {
            gt: query.gt,
            lt: query.lt,
            limit: query.limit,
        }
    }
}

impl FromJsonValue<ServiceListQuery> for ServiceListQuery {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceListResponse {
    pub meta: ServiceQuery,
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceCreateBody {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(url)]
    pub url: String,
}

impl FromJsonValue<ServiceCreateBody> for ServiceCreateBody {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceReadResponse {
    pub data: Service,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceUpdateBody {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
}

impl FromJsonValue<ServiceUpdateBody> for ServiceUpdateBody {}

// User types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserListQuery {
    #[validate(custom = "validate::id")]
    pub gt: Option<String>,
    #[validate(custom = "validate::id")]
    pub lt: Option<String>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<i64>,
    #[validate(email)]
    pub email_eq: Option<String>,
}

impl FromJsonValue<UserListQuery> for UserListQuery {}

impl From<UserListQuery> for UserQuery {
    fn from(query: UserListQuery) -> UserQuery {
        UserQuery {
            gt: query.gt,
            lt: query.lt,
            limit: query.limit,
            email_eq: query.email_eq,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserListResponse {
    pub meta: UserQuery,
    pub data: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateBody {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::password")]
    pub password: Option<String>,
}

impl FromJsonValue<UserCreateBody> for UserCreateBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreateResponse {
    pub meta: AuthPasswordMeta,
    pub data: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReadResponse {
    pub data: User,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserUpdateBody {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
}

impl FromJsonValue<UserUpdateBody> for UserUpdateBody {}
