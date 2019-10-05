//! # API Types
use crate::{
    ApiValidate, ApiValidateRequest, ApiValidateRequestQuery, Audit, AuditData, AuditList,
    AuditListCreatedGe, AuditListCreatedLe, AuditListCreatedLeAndGe, Core, Key, KeyList, KeyType,
    Service, ServiceCreate, ServiceList, ServiceUpdate, User, UserCreate, UserKey, UserList,
    UserPasswordMeta, UserToken, UserTokenAccess,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

// -----------
// Audit Types
// -----------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditListRequest {
    pub ge: Option<DateTime<Utc>>,
    pub le: Option<DateTime<Utc>>,
    #[validate(custom = "ApiValidate::limit")]
    pub limit: Option<i64>,
    pub offset_id: Option<Uuid>,
    #[serde(rename = "type")]
    #[validate(custom = "ApiValidate::audit_type_vec")]
    pub type_: Option<Vec<String>>,
    pub service_id: Option<Vec<Uuid>>,
    pub user_id: Option<Vec<Uuid>>,
}

impl ApiValidateRequest<AuditListRequest> for AuditListRequest {}
impl ApiValidateRequestQuery<AuditListRequest> for AuditListRequest {}

impl From<AuditListRequest> for AuditList {
    fn from(query: AuditListRequest) -> Self {
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

impl From<AuditList> for AuditListRequest {
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
    pub meta: AuditListRequest,
    pub data: Vec<Audit>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditCreateRequest {
    #[serde(alias = "type")]
    #[validate(custom = "ApiValidate::audit_type")]
    pub type_: String,
    pub data: Value,
    pub user_id: Option<Uuid>,
    pub user_key_id: Option<Uuid>,
}

impl ApiValidateRequest<AuditCreateRequest> for AuditCreateRequest {}

impl AuditCreateRequest {
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
    #[validate(custom = "ApiValidate::audit_type")]
    pub type_: String,
    pub data: Value,
}

impl ApiValidateRequest<AuditDataRequest> for AuditDataRequest {}

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

// ---------
// Key Types
// ---------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyListRequest {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "ApiValidate::limit")]
    pub limit: Option<i64>,
}

impl ApiValidateRequest<KeyListRequest> for KeyListRequest {}
impl ApiValidateRequestQuery<KeyListRequest> for KeyListRequest {}

impl From<KeyListRequest> for KeyList {
    fn from(query: KeyListRequest) -> KeyList {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        match (query.gt, query.lt) {
            (Some(gt), Some(_lt)) => Self::IdGt(gt, limit),
            (Some(gt), None) => Self::IdGt(gt, limit),
            (None, Some(lt)) => Self::IdLt(lt, limit),
            (None, None) => Self::Limit(limit),
        }
    }
}

impl From<KeyList> for KeyListRequest {
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
    pub meta: KeyListRequest,
    pub data: Vec<Key>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyCreateRequest {
    pub is_enabled: bool,
    #[serde(rename = "type")]
    pub type_: KeyType,
    #[validate(custom = "ApiValidate::name")]
    pub name: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl ApiValidateRequest<KeyCreateRequest> for KeyCreateRequest {}

impl KeyCreateRequest {
    pub fn new<S1: Into<String>>(is_enabled: bool, type_: KeyType, name: S1) -> Self {
        Self {
            is_enabled,
            type_,
            name: name.into(),
            service_id: None,
            user_id: None,
        }
    }

    pub fn with_service_id<S1>(is_enabled: bool, type_: KeyType, name: S1, service_id: Uuid) -> Self
    where
        S1: Into<String>,
    {
        Self {
            is_enabled,
            type_,
            name: name.into(),
            service_id: Some(service_id),
            user_id: None,
        }
    }

    pub fn with_user_id<S1>(is_enabled: bool, type_: KeyType, name: S1, user_id: Uuid) -> Self
    where
        S1: Into<String>,
    {
        Self {
            is_enabled,
            type_,
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
pub struct KeyUpdateRequest {
    pub is_enabled: Option<bool>,
    #[validate(custom = "ApiValidate::name")]
    pub name: Option<String>,
}

impl ApiValidateRequest<KeyUpdateRequest> for KeyUpdateRequest {}

// -------------
// Service Types
// -------------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceListRequest {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "ApiValidate::limit")]
    pub limit: Option<i64>,
}

impl ApiValidateRequest<ServiceListRequest> for ServiceListRequest {}
impl ApiValidateRequestQuery<ServiceListRequest> for ServiceListRequest {}

impl From<ServiceListRequest> for ServiceList {
    fn from(query: ServiceListRequest) -> ServiceList {
        let limit = query.limit.unwrap_or_else(Core::default_limit);

        match (query.gt, query.lt) {
            (Some(gt), Some(_lt)) => Self::IdGt(gt, limit),
            (Some(gt), None) => Self::IdGt(gt, limit),
            (None, Some(lt)) => Self::IdLt(lt, limit),
            (None, None) => Self::Limit(limit),
        }
    }
}

impl From<ServiceList> for ServiceListRequest {
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
    pub meta: ServiceListRequest,
    pub data: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceCreateRequest {
    pub is_enabled: bool,
    #[validate(custom = "ApiValidate::name")]
    pub name: String,
    #[validate(url)]
    pub url: String,
    #[validate(url)]
    pub provider_local_url: Option<String>,
    #[validate(url)]
    pub provider_github_oauth2_url: Option<String>,
    #[validate(url)]
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl ApiValidateRequest<ServiceCreateRequest> for ServiceCreateRequest {}

impl ServiceCreateRequest {
    pub fn new<S1, S2>(is_enabled: bool, name: S1, url: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            url: url.into(),
            provider_local_url: None,
            provider_github_oauth2_url: None,
            provider_microsoft_oauth2_url: None,
        }
    }

    pub fn provider_local_url<S: Into<String>>(mut self, provider_local_url: S) -> Self {
        self.provider_local_url = Some(provider_local_url.into());
        self
    }

    pub fn provider_github_oauth2_url<S: Into<String>>(
        mut self,
        provider_github_oauth2_url: S,
    ) -> Self {
        self.provider_github_oauth2_url = Some(provider_github_oauth2_url.into());
        self
    }

    pub fn provider_microsoft_oauth2_url<S: Into<String>>(
        mut self,
        provider_microsoft_oauth2_url: S,
    ) -> Self {
        self.provider_microsoft_oauth2_url = Some(provider_microsoft_oauth2_url.into());
        self
    }
}

impl From<ServiceCreateRequest> for ServiceCreate {
    fn from(request: ServiceCreateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            url: request.url,
            provider_local_url: request.provider_local_url,
            provider_github_oauth2_url: request.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: request.provider_microsoft_oauth2_url,
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
pub struct ServiceUpdateRequest {
    pub is_enabled: Option<bool>,
    #[validate(custom = "ApiValidate::name")]
    pub name: Option<String>,
    #[validate(url)]
    pub url: Option<String>,
    #[validate(url)]
    pub provider_local_url: Option<String>,
    #[validate(url)]
    pub provider_github_oauth2_url: Option<String>,
    #[validate(url)]
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl ApiValidateRequest<ServiceUpdateRequest> for ServiceUpdateRequest {}

impl From<ServiceUpdateRequest> for ServiceUpdate {
    fn from(request: ServiceUpdateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            url: request.url,
            provider_local_url: request.provider_local_url,
            provider_github_oauth2_url: request.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: request.provider_microsoft_oauth2_url,
        }
    }
}

// ----------
// User Types
// ----------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserListRequest {
    pub gt: Option<Uuid>,
    pub lt: Option<Uuid>,
    #[validate(custom = "ApiValidate::limit")]
    pub limit: Option<i64>,
    #[validate(email)]
    pub email_eq: Option<String>,
}

impl ApiValidateRequest<UserListRequest> for UserListRequest {}
impl ApiValidateRequestQuery<UserListRequest> for UserListRequest {}

impl From<UserListRequest> for UserList {
    fn from(query: UserListRequest) -> UserList {
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

impl From<UserList> for UserListRequest {
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
    pub meta: UserListRequest,
    pub data: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateRequest {
    pub is_enabled: bool,
    #[validate(custom = "ApiValidate::name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "ApiValidate::locale")]
    pub locale: String,
    #[validate(custom = "ApiValidate::timezone")]
    pub timezone: String,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
    #[validate(custom = "ApiValidate::password")]
    pub password: Option<String>,
}

impl ApiValidateRequest<UserCreateRequest> for UserCreateRequest {}

impl UserCreateRequest {
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
            password_allow_reset: None,
            password_require_update: None,
            password: None,
        }
    }

    pub fn with_password<S1, S2, S3, S4, S5>(
        is_enabled: bool,
        name: S1,
        email: S2,
        locale: S3,
        timezone: S4,
        password_allow_reset: bool,
        password_require_update: bool,
        password: S5,
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
            password_allow_reset: Some(password_allow_reset),
            password_require_update: Some(password_require_update),
            password: Some(password.into()),
        }
    }
}

impl From<UserCreateRequest> for UserCreate {
    fn from(request: UserCreateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            email: request.email,
            locale: request.locale,
            timezone: request.timezone,
            password_allow_reset: request.password_allow_reset.unwrap_or(false),
            password_require_update: request.password_require_update.unwrap_or(false),
            password_hash: request.password,
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
pub struct UserUpdateRequest {
    pub is_enabled: Option<bool>,
    #[validate(custom = "ApiValidate::name")]
    pub name: Option<String>,
    #[validate(custom = "ApiValidate::locale")]
    pub locale: Option<String>,
    #[validate(custom = "ApiValidate::timezone")]
    pub timezone: Option<String>,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
}

impl ApiValidateRequest<UserUpdateRequest> for UserUpdateRequest {}

// --------------------
// Authentication Types
// --------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenRequest {
    #[validate(custom = "ApiValidate::token")]
    pub token: String,
    pub audit: Option<AuditDataRequest>,
}

impl ApiValidateRequest<AuthTokenRequest> for AuthTokenRequest {}

impl AuthTokenRequest {
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
pub struct AuthKeyRequest {
    #[validate(custom = "ApiValidate::key")]
    pub key: String,
    pub audit: Option<AuditDataRequest>,
}

impl ApiValidateRequest<AuthKeyRequest> for AuthKeyRequest {}

impl AuthKeyRequest {
    pub fn new<S: Into<String>>(key: S, audit: Option<AuditDataRequest>) -> Self {
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
pub struct AuthTotpRequest {
    pub user_id: Uuid,
    #[validate(custom = "ApiValidate::totp")]
    pub totp: String,
}

impl ApiValidateRequest<AuthTotpRequest> for AuthTotpRequest {}

impl AuthTotpRequest {
    pub fn new<S: Into<String>>(user_id: Uuid, totp: S) -> Self {
        Self {
            user_id,
            totp: totp.into(),
        }
    }
}

// --------------------------
// Authentication Local Types
// --------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthLoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
}

impl ApiValidateRequest<AuthLoginRequest> for AuthLoginRequest {}

impl AuthLoginRequest {
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
pub struct AuthResetPasswordRequest {
    #[validate(email)]
    pub email: String,
}

impl ApiValidateRequest<AuthResetPasswordRequest> for AuthResetPasswordRequest {}

impl AuthResetPasswordRequest {
    pub fn new<S1: Into<String>>(email: S1) -> Self {
        Self {
            email: email.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthResetPasswordConfirmRequest {
    #[validate(custom = "ApiValidate::token")]
    pub token: String,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
}

impl ApiValidateRequest<AuthResetPasswordConfirmRequest> for AuthResetPasswordConfirmRequest {}

impl AuthResetPasswordConfirmRequest {
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
pub struct AuthUpdateEmailRequest {
    #[validate(custom = "ApiValidate::key")]
    pub key: Option<String>,
    #[validate(custom = "ApiValidate::token")]
    pub token: Option<String>,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
    #[validate(email)]
    pub new_email: String,
}

impl ApiValidateRequest<AuthUpdateEmailRequest> for AuthUpdateEmailRequest {}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthUpdatePasswordRequest {
    #[validate(custom = "ApiValidate::key")]
    pub key: Option<String>,
    #[validate(custom = "ApiValidate::token")]
    pub token: Option<String>,
    #[validate(custom = "ApiValidate::password")]
    pub password: String,
    #[validate(custom = "ApiValidate::password")]
    pub new_password: String,
}

impl ApiValidateRequest<AuthUpdatePasswordRequest> for AuthUpdatePasswordRequest {}

// ---------------------------
// Authentication OAuth2 Types
// ---------------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthOauth2UrlResponse {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AuthOauth2CallbackRequest {
    #[validate(custom = "ApiValidate::token")]
    pub code: String,
    #[validate(custom = "ApiValidate::token")]
    pub state: String,
}

impl ApiValidateRequest<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}
impl ApiValidateRequestQuery<AuthOauth2CallbackRequest> for AuthOauth2CallbackRequest {}
