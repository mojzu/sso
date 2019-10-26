use crate::{
    api::{
        result_audit_diff, result_audit_err, result_audit_subject, validate, ApiResult,
        ValidateRequest, ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Driver, Key, KeyListFilter, KeyListQuery, KeyRead, KeyType,
    KeyWithValue, DEFAULT_LIMIT,
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

pub fn key_list(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: KeyListRequest,
) -> ApiResult<KeyListResponse> {
    KeyListRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyList);
    let (query, filter) = request.into_query_filter();

    let res = server_key::list(driver, &mut audit, key_value, &query, &filter);
    result_audit_err(driver, &audit, res).map(|data| KeyListResponse {
        meta: KeyListRequest::from_query_filter(query, filter),
        data,
    })
}

pub fn key_create(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: KeyCreateRequest,
) -> ApiResult<KeyCreateResponse> {
    KeyCreateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyCreate);

    let res = server_key::create(driver, &mut audit, key_value, request);
    result_audit_subject(driver, &audit, res).map(|key| KeyCreateResponse { data: key })
}

pub fn key_read(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    key_id: Uuid,
) -> ApiResult<KeyReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyRead);

    let res = server_key::read(driver, &mut audit, key_value, key_id);
    result_audit_err(driver, &audit, res).map(|data| KeyReadResponse { data })
}

pub fn key_update(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    key_id: Uuid,
    request: KeyUpdateRequest,
) -> ApiResult<KeyReadResponse> {
    KeyUpdateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyUpdate);

    let res = server_key::update(driver, &mut audit, key_value, key_id, request);
    result_audit_diff(driver, &audit, res).map(|key| KeyReadResponse { data: key })
}

pub fn key_delete(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    key_id: Uuid,
) -> ApiResult<()> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::KeyDelete);

    let res = server_key::delete(driver, &mut audit, key_value, key_id);
    result_audit_subject(driver, &audit, res).map(|_| ())
}

mod server_key {
    use super::*;
    use crate::{
        api::{ApiError, ApiResult},
        util::*,
        AuditBuilder, Driver, DriverError, Key, KeyCreate, KeyList, KeyListFilter, KeyListQuery,
        KeyUpdate, Service,
    };

    pub fn list(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        query: &KeyListQuery,
        filter: &KeyListFilter,
    ) -> ApiResult<Vec<Key>> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let list = KeyList {
            query,
            filter,
            service_id_mask: service.map(|s| s.id),
        };
        driver.key_list(&list).map_err(ApiError::BadRequest)
    }

    pub fn create(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: KeyCreateRequest,
    ) -> ApiResult<KeyWithValue> {
        // If service ID is some, root key is required to create service keys.
        match request.service_id {
            Some(service_id) => {
                key_root_authenticate(driver, audit, key_value)
                    .map_err(ApiError::Unauthorised)
                    .and_then(|_| {
                        match request.user_id {
                            // User ID is defined, creating user key for service.
                            Some(user_id) => driver.key_create(&KeyCreate::user(
                                request.is_enabled,
                                request.type_,
                                request.name,
                                service_id,
                                user_id,
                            )),
                            // Creating service key.
                            None => driver.key_create(&KeyCreate::service(
                                request.is_enabled,
                                request.name,
                                service_id,
                            )),
                        }
                        .map_err(ApiError::BadRequest)
                    })
            }
            None => {
                key_service_authenticate(driver, audit, key_value)
                    .map_err(ApiError::Unauthorised)
                    .and_then(|service| {
                        match request.user_id {
                            // User ID is defined, creating user key for service.
                            Some(user_id) => driver.key_create(&KeyCreate::user(
                                request.is_enabled,
                                request.type_,
                                request.name,
                                service.id,
                                user_id,
                            )),
                            // Service cannot create service keys.
                            None => Err(DriverError::ServiceCannotCreateServiceKey),
                        }
                        .map_err(ApiError::BadRequest)
                    })
            }
        }
    }

    pub fn read(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        key_id: Uuid,
    ) -> ApiResult<Key> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        read_inner(driver, service.as_ref(), key_id)
    }

    pub fn update(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        key_id: Uuid,
        request: KeyUpdateRequest,
    ) -> ApiResult<(Key, Key)> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let previous_key = read_inner(driver, service.as_ref(), key_id)?;
        let key = driver
            .key_update(
                &key_id,
                &KeyUpdate {
                    is_enabled: request.is_enabled,
                    is_revoked: None,
                    name: request.name,
                },
            )
            .map_err(ApiError::BadRequest)?;
        Ok((previous_key, key))
    }

    pub fn delete(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        key_id: Uuid,
    ) -> ApiResult<Key> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let key = read_inner(driver, service.as_ref(), key_id)?;
        driver
            .key_delete(&key_id)
            .map_err(ApiError::BadRequest)
            .map(|_| key)
    }

    fn read_inner(driver: &dyn Driver, service: Option<&Service>, key_id: Uuid) -> ApiResult<Key> {
        let read = KeyRead::Id(key_id, service.map(|x| x.id));
        driver
            .key_read(&read)
            .map_err(ApiError::BadRequest)?
            .ok_or_else(|| ApiError::NotFound(DriverError::KeyNotFound))
            .map(|x| x.into())
    }
}
