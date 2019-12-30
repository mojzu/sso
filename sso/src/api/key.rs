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
