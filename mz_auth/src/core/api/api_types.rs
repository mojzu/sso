use crate::{
    ApiValidate, ApiValidateRequest, ApiValidateRequestQuery, Audit, AuditData, AuditList,
    AuditListCreatedGe, AuditListCreatedLe, AuditListCreatedLeAndGe, Core, UserToken,
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

// --------------------
// Authentication Types
// --------------------

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthTokenResponse {
    pub data: UserToken,
}

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
