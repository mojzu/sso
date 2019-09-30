//! # Server API Types
use crate::{
    Audit, AuditData, AuditList, AuditListCreatedGe, AuditListCreatedLe, AuditListCreatedLeAndGe,
    Core, Key, KeyList, ServerValidate, ServerValidateFromStr, ServerValidateFromValue, Service,
    ServiceList, User, UserKey, UserList, UserPasswordMeta, UserToken, UserTokenAccess,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

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
    pub const TOTP: &str = "/totp";
    pub const AUDIT: &str = "/audit";
    pub const SERVICE: &str = "/service";
    pub const USER: &str = "/user";
}

/// Route definitions.
pub mod route {
    use std::fmt::Display;

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
    pub const AUTH_TOTP: &str = "/v1/auth/totp";
    pub const AUDIT: &str = "/v1/audit";
    pub const KEY: &str = "/v1/key";
    pub const SERVICE: &str = "/v1/service";
    pub const USER: &str = "/v1/user";

    pub fn audit_id<T: Display>(id: T) -> String {
        format!("{}/{}", AUDIT, id)
    }

    pub fn key_id<T: Display>(id: T) -> String {
        format!("{}/{}", KEY, id)
    }

    pub fn service_id<T: Display>(id: T) -> String {
        format!("{}/{}", SERVICE, id)
    }

    pub fn user_id<T: Display>(id: T) -> String {
        format!("{}/{}", USER, id)
    }
}

// Audit types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditListQuery {
    pub ge: Option<DateTime<Utc>>,
    pub le: Option<DateTime<Utc>>,
    #[validate(custom = "ServerValidate::limit")]
    pub limit: Option<i64>,
    pub offset_id: Option<Uuid>,
    #[serde(rename = "type")]
    #[validate(custom = "ServerValidate::audit_type_vec")]
    pub type_: Option<Vec<String>>,
    pub service_id: Option<Vec<Uuid>>,
    pub user_id: Option<Vec<Uuid>>,
}

impl ServerValidateFromStr<AuditListQuery> for AuditListQuery {}

impl From<AuditListQuery> for AuditList {
    fn from(query: AuditListQuery) -> Self {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        match (query.ge, query.le) {
            (Some(ge), Some(le)) => Self::CreatedLeAndGe(AuditListCreatedLeAndGe {
                ge,
                le,
                limit,
                offset_id: query.offset_id,
                type_: query.type_,
                service_id: query.service_id,
                user_id: query.user_id,
            }),
            (Some(ge), None) => Self::CreatedGe(AuditListCreatedGe {
                ge,
                limit,
                offset_id: query.offset_id,
                type_: query.type_,
                service_id: query.service_id,
                user_id: query.user_id,
            }),
            (None, Some(le)) => Self::CreatedLe(AuditListCreatedLe {
                le,
                limit,
                offset_id: query.offset_id,
                type_: query.type_,
                service_id: query.service_id,
                user_id: query.user_id,
            }),
            (None, None) => Self::CreatedLe(AuditListCreatedLe {
                le: Utc::now(),
                limit,
                offset_id: query.offset_id,
                type_: query.type_,
                service_id: query.service_id,
                user_id: query.user_id,
            }),
        }
    }
}

impl From<AuditList> for AuditListQuery {
    fn from(list: AuditList) -> Self {
        match list {
            AuditList::CreatedLe(l) => Self {
                ge: None,
                le: Some(l.le),
                limit: Some(l.limit),
                offset_id: l.offset_id,
                type_: l.type_,
                service_id: l.service_id,
                user_id: l.user_id,
            },
            AuditList::CreatedGe(l) => Self {
                ge: Some(l.ge),
                le: None,
                limit: Some(l.limit),
                offset_id: l.offset_id,
                type_: l.type_,
                service_id: l.service_id,
                user_id: l.user_id,
            },
            AuditList::CreatedLeAndGe(l) => Self {
                ge: Some(l.ge),
                le: Some(l.le),
                limit: Some(l.limit),
                offset_id: l.offset_id,
                type_: l.type_,
                service_id: l.service_id,
                user_id: l.user_id,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditListResponse {
    pub meta: AuditListQuery,
    pub data: Vec<Audit>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditCreateBody {
    #[serde(alias = "type")]
    #[validate(custom = "ServerValidate::audit_type")]
    pub type_: String,
    pub data: Value,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl ServerValidateFromValue<AuditCreateBody> for AuditCreateBody {}

impl AuditCreateBody {
    pub fn new<T1>(type_: T1, data: Value, user_id: Option<Uuid>, user_key_id: Option<Uuid>) -> Self
    where
        T1: Into<String>,
    {
        Self {
            type_: type_.into(),
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
    #[serde(alias = "type")]
    #[validate(custom = "ServerValidate::audit_type")]
    pub type_: String,
    pub data: Value,
}

impl ServerValidateFromValue<AuditDataRequest> for AuditDataRequest {}

impl AuditDataRequest {
    pub fn new<T1>(type_: T1, data: Value) -> Self
    where
        T1: Into<String>,
    {
        Self {
            type_: type_.into(),
            data,
        }
    }
}

impl From<AuditDataRequest> for AuditData {
    fn from(data: AuditDataRequest) -> AuditData {
        AuditData {
            type_: data.type_,
            data: data.data,
        }
    }
}

// Authentication types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenBody {
    #[validate(custom = "ServerValidate::token")]
    pub token: String,
    pub audit: Option<AuditDataRequest>,
}

impl ServerValidateFromValue<AuthTokenBody> for AuthTokenBody {}

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
pub struct AuthTokenAccessResponse {
    pub data: UserTokenAccess,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthKeyBody {
    #[validate(custom = "ServerValidate::key")]
    pub key: String,
    pub audit: Option<AuditDataRequest>,
}

impl ServerValidateFromValue<AuthKeyBody> for AuthKeyBody {}

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

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTotpBody {
    pub key_id: Uuid,
    #[validate(custom = "ServerValidate::totp")]
    pub totp: String,
}

impl ServerValidateFromValue<AuthTotpBody> for AuthTotpBody {}

// Authentication local provider types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginBody {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "ServerValidate::password")]
    pub password: String,
}

impl ServerValidateFromValue<AuthLoginBody> for AuthLoginBody {}

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
    pub meta: UserPasswordMeta,
    pub data: UserToken,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthResetPasswordBody {
    #[validate(email)]
    pub email: String,
}

impl ServerValidateFromValue<AuthResetPasswordBody> for AuthResetPasswordBody {}

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
    #[validate(custom = "ServerValidate::token")]
    pub token: String,
    #[validate(custom = "ServerValidate::password")]
    pub password: String,
}

impl ServerValidateFromValue<AuthResetPasswordConfirmBody> for AuthResetPasswordConfirmBody {}

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
    pub meta: UserPasswordMeta,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdateEmailBody {
    #[validate(custom = "ServerValidate::key")]
    pub key: Option<String>,
    #[validate(custom = "ServerValidate::token")]
    pub token: Option<String>,
    #[validate(custom = "ServerValidate::password")]
    pub password: String,
    #[validate(email)]
    pub new_email: String,
}

impl ServerValidateFromValue<AuthUpdateEmailBody> for AuthUpdateEmailBody {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdatePasswordBody {
    #[validate(custom = "ServerValidate::key")]
    pub key: Option<String>,
    #[validate(custom = "ServerValidate::token")]
    pub token: Option<String>,
    #[validate(custom = "ServerValidate::password")]
    pub password: String,
    #[validate(custom = "ServerValidate::password")]
    pub new_password: String,
}

impl ServerValidateFromValue<AuthUpdatePasswordBody> for AuthUpdatePasswordBody {}

// Authentication OAuth2 provider types.

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthOauth2UrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthOauth2CallbackQuery {
    #[validate(custom = "ServerValidate::token")]
    pub code: String,
    #[validate(custom = "ServerValidate::token")]
    pub state: String,
}

impl ServerValidateFromStr<AuthOauth2CallbackQuery> for AuthOauth2CallbackQuery {}

// Key types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyListQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "ServerValidate::limit")]
    pub limit: Option<i64>,
}

impl ServerValidateFromStr<KeyListQuery> for KeyListQuery {}

impl From<KeyListQuery> for KeyList {
    fn from(query: KeyListQuery) -> KeyList {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        match (query.gt, query.lt) {
            (Some(gt), Some(_lt)) => Self::IdGt(gt, limit),
            (Some(gt), None) => Self::IdGt(gt, limit),
            (None, Some(lt)) => Self::IdLt(lt, limit),
            (None, None) => Self::Limit(limit),
        }
    }
}

impl From<KeyList> for KeyListQuery {
    fn from(list: KeyList) -> Self {
        match list {
            KeyList::Limit(limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
            },
            KeyList::IdGt(gt, limit) => Self {
                gt: Some(gt),
                lt: None,
                limit: Some(limit),
            },
            KeyList::IdLt(lt, limit) => Self {
                gt: None,
                lt: Some(lt),
                limit: Some(limit),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyListResponse {
    pub meta: KeyListQuery,
    pub data: Vec<Key>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyCreateBody {
    pub is_enabled: bool,
    pub allow_key: bool,
    pub allow_token: bool,
    pub allow_totp: bool,
    #[validate(custom = "ServerValidate::name")]
    pub name: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl ServerValidateFromValue<KeyCreateBody> for KeyCreateBody {}

impl KeyCreateBody {
    pub fn new<S1: Into<String>>(
        is_enabled: bool,
        allow_key: bool,
        allow_token: bool,
        allow_totp: bool,
        name: S1,
    ) -> Self {
        Self {
            is_enabled,
            allow_key,
            allow_token,
            allow_totp,
            name: name.into(),
            service_id: None,
            user_id: None,
        }
    }

    pub fn with_service_id<S1>(
        is_enabled: bool,
        allow_key: bool,
        allow_token: bool,
        allow_totp: bool,
        name: S1,
        service_id: Uuid,
    ) -> Self
    where
        S1: Into<String>,
    {
        Self {
            is_enabled,
            allow_key,
            allow_token,
            allow_totp,
            name: name.into(),
            service_id: Some(service_id),
            user_id: None,
        }
    }

    pub fn with_user_id<S1>(
        is_enabled: bool,
        allow_key: bool,
        allow_token: bool,
        allow_totp: bool,
        name: S1,
        user_id: Uuid,
    ) -> Self
    where
        S1: Into<String>,
    {
        Self {
            is_enabled,
            allow_key,
            allow_token,
            allow_totp,
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
    pub allow_key: Option<bool>,
    pub allow_token: Option<bool>,
    pub allow_totp: Option<bool>,
    #[validate(custom = "ServerValidate::name")]
    pub name: Option<String>,
}

impl ServerValidateFromValue<KeyUpdateBody> for KeyUpdateBody {}

// Service types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceListQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "ServerValidate::limit")]
    pub limit: Option<i64>,
}

impl ServerValidateFromStr<ServiceListQuery> for ServiceListQuery {}

impl From<ServiceListQuery> for ServiceList {
    fn from(query: ServiceListQuery) -> ServiceList {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        match (query.gt, query.lt) {
            (Some(gt), Some(_lt)) => Self::IdGt(gt, limit),
            (Some(gt), None) => Self::IdGt(gt, limit),
            (None, Some(lt)) => Self::IdLt(lt, limit),
            (None, None) => Self::Limit(limit),
        }
    }
}

impl From<ServiceList> for ServiceListQuery {
    fn from(list: ServiceList) -> Self {
        match list {
            ServiceList::Limit(limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
            },
            ServiceList::IdGt(gt, limit) => Self {
                gt: Some(gt),
                lt: None,
                limit: Some(limit),
            },
            ServiceList::IdLt(lt, limit) => Self {
                gt: None,
                lt: Some(lt),
                limit: Some(limit),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceListResponse {
    pub meta: ServiceListQuery,
    pub data: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceCreateBody {
    pub is_enabled: bool,
    #[validate(custom = "ServerValidate::name")]
    pub name: String,
    #[validate(url)]
    pub url: String,
}

impl ServerValidateFromValue<ServiceCreateBody> for ServiceCreateBody {}

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
    #[validate(custom = "ServerValidate::name")]
    pub name: Option<String>,
}

impl ServerValidateFromValue<ServiceUpdateBody> for ServiceUpdateBody {}

// User types.

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserListQuery {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "ServerValidate::limit")]
    pub limit: Option<i64>,
    #[validate(email)]
    pub email_eq: Option<String>,
}

impl ServerValidateFromStr<UserListQuery> for UserListQuery {}

impl From<UserListQuery> for UserList {
    fn from(query: UserListQuery) -> UserList {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        if let Some(email_eq) = query.email_eq {
            return UserList::EmailEq(email_eq, limit);
        }
        match (query.gt, query.lt) {
            (Some(gt), Some(_lt)) => Self::IdGt(gt, limit),
            (Some(gt), None) => Self::IdGt(gt, limit),
            (None, Some(lt)) => Self::IdLt(lt, limit),
            (None, None) => Self::Limit(limit),
        }
    }
}

impl From<UserList> for UserListQuery {
    fn from(list: UserList) -> Self {
        match list {
            UserList::Limit(limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                email_eq: None,
            },
            UserList::IdGt(gt, limit) => Self {
                gt: Some(gt),
                lt: None,
                limit: Some(limit),
                email_eq: None,
            },
            UserList::IdLt(lt, limit) => Self {
                gt: None,
                lt: Some(lt),
                limit: Some(limit),
                email_eq: None,
            },
            UserList::EmailEq(email_eq, limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                email_eq: Some(email_eq),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserListResponse {
    pub meta: UserListQuery,
    pub data: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateBody {
    pub is_enabled: bool,
    #[validate(custom = "ServerValidate::name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "ServerValidate::locale")]
    pub locale: String,
    #[validate(custom = "ServerValidate::timezone")]
    pub timezone: String,
    #[validate(custom = "ServerValidate::password")]
    pub password: Option<String>,
    pub password_update_required: Option<bool>,
}

impl ServerValidateFromValue<UserCreateBody> for UserCreateBody {}

impl UserCreateBody {
    pub fn new<S1, S2, S3, S4>(
        is_enabled: bool,
        name: S1,
        email: S2,
        locale: S3,
        timezone: S4,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            email: email.into(),
            locale: locale.into(),
            timezone: timezone.into(),
            password: None,
            password_update_required: None,
        }
    }

    pub fn with_password<S1, S2, S3, S4, S5>(
        is_enabled: bool,
        name: S1,
        email: S2,
        locale: S3,
        timezone: S4,
        password: S5,
        password_update_required: bool,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
        S5: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            email: email.into(),
            locale: locale.into(),
            timezone: timezone.into(),
            password: Some(password.into()),
            password_update_required: Some(password_update_required),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreateResponse {
    pub meta: UserPasswordMeta,
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
    #[validate(custom = "ServerValidate::name")]
    pub name: Option<String>,
    #[validate(custom = "ServerValidate::locale")]
    pub locale: Option<String>,
    #[validate(custom = "ServerValidate::timezone")]
    pub timezone: Option<String>,
    pub password_update_required: Option<bool>,
}

impl ServerValidateFromValue<UserUpdateBody> for UserUpdateBody {}
