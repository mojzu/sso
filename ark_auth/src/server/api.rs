use crate::{
    core::{
        Audit, AuditData, AuditQuery, Key, KeyQuery, Service, ServiceQuery, User, UserKey,
        UserQuery, UserToken, UserTokenPartial,
    },
    server::{validate, FromJsonValue},
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

fn datetime_from_string(value: Option<String>) -> Option<DateTime<Utc>> {
    value.map(|x| {
        DateTime::parse_from_rfc3339(&x)
            .unwrap()
            .with_timezone(&Utc)
    })
}

fn i64_from_string(value: Option<String>) -> Option<i64> {
    value.map(|x| x.parse::<i64>().unwrap())
}

/// Path definitions.
pub mod path {
    pub const NONE: &str = "";
    pub const ID: &str = "/{id}";
    pub const V1: &str = "/v1";
    pub const PING: &str = "/ping";
    pub const METRICS: &str = "/metrics";
    pub const AUTH: &str = "/auth";
    pub const PROVIDER: &str = "/provider";
    pub const LOCAL: &str = "/local";
    pub const LOGIN: &str = "/login";
    pub const RESET_PASSWORD: &str = "/reset-password";
    pub const UPDATE_EMAIL: &str = "/update-email";
    pub const UPDATE_PASSWORD: &str = "/update-password";
    pub const CONFIRM: &str = "/confirm";
    pub const GITHUB: &str = "/github";
    pub const MICROSOFT: &str = "/microsoft";
    pub const OAUTH2: &str = "/oauth2";
    pub const KEY: &str = "/key";
    pub const TOKEN: &str = "/token";
    pub const VERIFY: &str = "/verify";
    pub const REFRESH: &str = "/refresh";
    pub const REVOKE: &str = "/revoke";
    pub const AUDIT: &str = "/audit";
    pub const SERVICE: &str = "/service";
    pub const USER: &str = "/user";
}

/// Route definitions.
pub mod route {
    pub const PING: &str = "/v1/ping";
    pub const METRICS: &str = "/v1/metrics";
    pub const AUTH_LOCAL_LOGIN: &str = "/v1/auth/provider/local/login";
    pub const AUTH_LOCAL_RESET_PASSWORD: &str = "/v1/auth/provider/local/reset-password";
    pub const AUTH_LOCAL_RESET_PASSWORD_CONFIRM: &str =
        "/v1/auth/provider/local/reset-password/confirm";
    pub const AUTH_LOCAL_UPDATE_EMAIL: &str = "/v1/auth/provider/local/update-email";
    pub const AUTH_LOCAL_UPDATE_EMAIL_REVOKE: &str = "/v1/auth/provider/local/update-email/revoke";
    pub const AUTH_LOCAL_UPDATE_PASSWORD: &str = "/v1/auth/provider/local/update-password";
    pub const AUTH_LOCAL_UPDATE_PASSWORD_REVOKE: &str =
        "/v1/auth/provider/local/update-password/revoke";
    pub const AUTH_GITHUB_OAUTH2: &str = "/v1/auth/provider/github/oauth2";
    pub const AUTH_MICROSOFT_OAUTH2: &str = "/v1/auth/provider/microsoft/oauth2";
    pub const AUTH_KEY_VERIFY: &str = "/v1/auth/key/verify";
    pub const AUTH_KEY_REVOKE: &str = "/v1/auth/key/revoke";
    pub const AUTH_TOKEN_VERIFY: &str = "/v1/auth/token/verify";
    pub const AUTH_TOKEN_REFRESH: &str = "/v1/auth/token/refresh";
    pub const AUTH_TOKEN_REVOKE: &str = "/v1/auth/token/revoke";
    pub const AUDIT: &str = "/v1/audit";
    pub const KEY: &str = "/v1/key";
    pub const SERVICE: &str = "/v1/service";
    pub const USER: &str = "/v1/user";

    pub fn audit_id(id: &str) -> String {
        format!("{}/{}", AUDIT, id)
    }

    pub fn key_id(id: &str) -> String {
        format!("{}/{}", KEY, id)
    }

    pub fn service_id(id: &str) -> String {
        format!("{}/{}", SERVICE, id)
    }

    pub fn user_id(id: &str) -> String {
        format!("{}/{}", USER, id)
    }
}

// Audit types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditListQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    pub created_gte: Option<DateTime<Utc>>,
    pub created_lte: Option<DateTime<Utc>>,
    pub offset_id: Option<Uuid>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<String>,
}

impl FromJsonValue<AuditListQuery> for AuditListQuery {}

impl From<AuditListQuery> for AuditQuery {
    fn from(query: AuditListQuery) -> AuditQuery {
        AuditQuery {
            gt: query.gt,
            lt: query.lt,
            created_gte: query.created_gte,
            created_lte: query.created_lte,
            offset_id: query.offset_id,
            limit: i64_from_string(query.limit),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditListResponse {
    pub meta: AuditQuery,
    pub data: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditCreateBody {
    #[validate(custom = "validate::path")]
    pub path: String,
    pub data: Value,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl FromJsonValue<AuditCreateBody> for AuditCreateBody {}

impl AuditCreateBody {
    pub fn new<T1>(path: T1, data: Value, user_id: Option<Uuid>, user_key_id: Option<Uuid>) -> Self
    where
        T1: Into<String>,
    {
        Self {
            path: path.into(),
            data,
            user_id,
            user_key_id,
        }
    }
}

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

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditDataRequest {
    #[validate(custom = "validate::path")]
    pub path: String,
    pub data: Value,
}

impl FromJsonValue<AuditDataRequest> for AuditDataRequest {}

impl AuditDataRequest {
    pub fn new<T1>(path: T1, data: Value) -> Self
    where
        T1: Into<String>,
    {
        Self {
            path: path.into(),
            data,
        }
    }
}

impl From<AuditDataRequest> for AuditData {
    fn from(data: AuditDataRequest) -> AuditData {
        AuditData {
            path: data.path,
            data: data.data,
        }
    }
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

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenBody {
    #[validate(custom = "validate::token")]
    pub token: String,
    pub audit: Option<AuditDataRequest>,
}

impl FromJsonValue<AuthTokenBody> for AuthTokenBody {}

impl AuthTokenBody {
    pub fn new<S1: Into<String>>(token: S1, audit: Option<AuditDataRequest>) -> Self {
        Self {
            token: token.into(),
            audit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenResponse {
    pub data: UserToken,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenPartialResponse {
    pub data: UserTokenPartial,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyBody {
    #[validate(custom = "validate::key")]
    pub key: String,
    pub audit: Option<AuditDataRequest>,
}

impl FromJsonValue<AuthKeyBody> for AuthKeyBody {}

impl AuthKeyBody {
    pub fn new<S1: Into<String>>(key: S1, audit: Option<AuditDataRequest>) -> Self {
        Self {
            key: key.into(),
            audit,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyResponse {
    pub data: UserKey,
}

// Authentication local provider types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::password")]
    pub password: String,
}

impl FromJsonValue<AuthLoginBody> for AuthLoginBody {}

impl AuthLoginBody {
    pub fn new<S1, S2>(email: S1, password: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginResponse {
    pub meta: AuthPasswordMeta,
    pub data: UserToken,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthResetPasswordBody {
    #[validate(email)]
    pub email: String,
}

impl FromJsonValue<AuthResetPasswordBody> for AuthResetPasswordBody {}

impl AuthResetPasswordBody {
    pub fn new<S1: Into<String>>(email: S1) -> Self {
        Self {
            email: email.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthResetPasswordConfirmBody {
    #[validate(custom = "validate::token")]
    pub token: String,
    #[validate(custom = "validate::password")]
    pub password: String,
}

impl FromJsonValue<AuthResetPasswordConfirmBody> for AuthResetPasswordConfirmBody {}

impl AuthResetPasswordConfirmBody {
    pub fn new<S1, S2>(token: S1, password: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            token: token.into(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthPasswordMetaResponse {
    pub meta: AuthPasswordMeta,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdateEmailBody {
    #[validate(custom = "validate::key")]
    pub key: Option<String>,
    #[validate(custom = "validate::token")]
    pub token: Option<String>,
    #[validate(custom = "validate::password")]
    pub password: String,
    #[validate(email)]
    pub new_email: String,
}

impl FromJsonValue<AuthUpdateEmailBody> for AuthUpdateEmailBody {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdatePasswordBody {
    #[validate(custom = "validate::key")]
    pub key: Option<String>,
    #[validate(custom = "validate::token")]
    pub token: Option<String>,
    #[validate(custom = "validate::password")]
    pub password: String,
    #[validate(custom = "validate::password")]
    pub new_password: String,
}

impl FromJsonValue<AuthUpdatePasswordBody> for AuthUpdatePasswordBody {}

// Authentication OAuth2 provider types.

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthOauth2UrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthOauth2CallbackQuery {
    #[validate(custom = "validate::token")]
    pub code: String,
    #[validate(custom = "validate::token")]
    pub state: String,
}

impl FromJsonValue<AuthOauth2CallbackQuery> for AuthOauth2CallbackQuery {}

// Key types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyListQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<String>,
}

impl FromJsonValue<KeyListQuery> for KeyListQuery {}

impl From<KeyListQuery> for KeyQuery {
    fn from(query: KeyListQuery) -> KeyQuery {
        KeyQuery {
            gt: query.gt,
            lt: query.lt,
            limit: i64_from_string(query.limit),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyListResponse {
    pub meta: KeyQuery,
    pub data: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyCreateBody {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl FromJsonValue<KeyCreateBody> for KeyCreateBody {}

impl KeyCreateBody {
    pub fn new<S1: Into<String>>(is_enabled: bool, name: S1) -> Self {
        Self {
            is_enabled,
            name: name.into(),
            service_id: None,
            user_id: None,
        }
    }

    pub fn with_service_id<S1, S2>(is_enabled: bool, name: S1, service_id: Uuid) -> Self
    where
        S1: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            service_id: Some(service_id),
            user_id: None,
        }
    }

    pub fn with_user_id<S1, S2>(is_enabled: bool, name: S1, user_id: Uuid) -> Self
    where
        S1: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            service_id: None,
            user_id: Some(user_id),
        }
    }
}

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
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<String>,
}

impl From<ServiceListQuery> for ServiceQuery {
    fn from(query: ServiceListQuery) -> ServiceQuery {
        ServiceQuery {
            gt: query.gt,
            lt: query.lt,
            limit: i64_from_string(query.limit),
        }
    }
}

impl FromJsonValue<ServiceListQuery> for ServiceListQuery {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceListResponse {
    pub meta: ServiceQuery,
    pub data: Vec<Uuid>,
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

impl ServiceCreateBody {
    pub fn new<S1, S2>(is_enabled: bool, name: S1, url: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            url: url.into(),
        }
    }
}

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
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "validate::limit")]
    pub limit: Option<String>,
    #[validate(email)]
    pub email_eq: Option<String>,
}

impl FromJsonValue<UserListQuery> for UserListQuery {}

impl From<UserListQuery> for UserQuery {
    fn from(query: UserListQuery) -> UserQuery {
        UserQuery {
            gt: query.gt,
            lt: query.lt,
            limit: i64_from_string(query.limit),
            email_eq: query.email_eq,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserListResponse {
    pub meta: UserQuery,
    pub data: Vec<Uuid>,
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

impl UserCreateBody {
    pub fn new<S1, S2>(is_enabled: bool, name: S1, email: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            email: email.into(),
            password: None,
        }
    }

    pub fn with_password<S1, S2, S3>(is_enabled: bool, name: S1, email: S2, password: S3) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            email: email.into(),
            password: Some(password.into()),
        }
    }
}

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
