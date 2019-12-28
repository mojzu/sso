use crate::{
    api::{validate, ValidateRequest, ValidateRequestQuery},
    Key, KeyListFilter, KeyListQuery, KeyType, KeyWithValue, DEFAULT_LIMIT,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct KeyListRequest {
    #[builder(default = "None")]
    gt: Option<Uuid>,

    #[builder(default = "None")]
    lt: Option<Uuid>,

    #[validate(custom = "validate::limit")]
    #[builder(default = "None")]
    limit: Option<i64>,

    #[builder(default = "None")]
    id: Option<Vec<Uuid>>,

    #[builder(default = "None")]
    is_enabled: Option<bool>,

    #[builder(default = "None")]
    is_revoked: Option<bool>,

    #[serde(rename = "type")]
    #[builder(default = "None")]
    type_: Option<Vec<KeyType>>,

    #[builder(default = "None")]
    service_id: Option<Vec<Uuid>>,

    #[builder(default = "None")]
    user_id: Option<Vec<Uuid>>,
}

impl ValidateRequest<KeyListRequest> for KeyListRequest {}
impl ValidateRequestQuery<KeyListRequest> for KeyListRequest {}

impl KeyListRequest {
    pub fn into_query_filter(self) -> (KeyListQuery, KeyListFilter) {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        let query = match (self.gt, self.lt) {
            (Some(gt), Some(_lt)) => KeyListQuery::IdGt(gt, limit),
            (Some(gt), None) => KeyListQuery::IdGt(gt, limit),
            (None, Some(lt)) => KeyListQuery::IdLt(lt, limit),
            (None, None) => KeyListQuery::Limit(limit),
        };

        let filter = KeyListFilter {
            id: self.id,
            is_enabled: self.is_enabled,
            is_revoked: self.is_revoked,
            type_: self.type_,
            service_id: self.service_id,
            user_id: self.user_id,
        };

        (query, filter)
    }

    pub fn from_query_filter(query: KeyListQuery, filter: KeyListFilter) -> Self {
        match query {
            KeyListQuery::Limit(limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                id: filter.id,
                is_enabled: filter.is_enabled,
                is_revoked: filter.is_revoked,
                type_: filter.type_,
                service_id: filter.service_id,
                user_id: filter.user_id,
            },
            KeyListQuery::IdGt(gt, limit) => Self {
                gt: Some(gt),
                lt: None,
                limit: Some(limit),
                id: filter.id,
                is_enabled: filter.is_enabled,
                is_revoked: filter.is_revoked,
                type_: filter.type_,
                service_id: filter.service_id,
                user_id: filter.user_id,
            },
            KeyListQuery::IdLt(lt, limit) => Self {
                gt: None,
                lt: Some(lt),
                limit: Some(limit),
                id: filter.id,
                is_enabled: filter.is_enabled,
                is_revoked: filter.is_revoked,
                type_: filter.type_,
                service_id: filter.service_id,
                user_id: filter.user_id,
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
    #[validate(custom = "validate::name")]
    pub name: String,
    pub service_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl ValidateRequest<KeyCreateRequest> for KeyCreateRequest {}

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
pub struct KeyCreateResponse {
    pub data: KeyWithValue,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct KeyReadRequest {
    pub user_id: Option<Uuid>,
}

impl ValidateRequest<KeyReadRequest> for KeyReadRequest {}
impl ValidateRequestQuery<KeyReadRequest> for KeyReadRequest {}

impl KeyReadRequest {
    pub fn new(user_id: Option<Uuid>) -> Self {
        Self { user_id }
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
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
}

impl ValidateRequest<KeyUpdateRequest> for KeyUpdateRequest {}
