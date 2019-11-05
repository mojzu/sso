use crate::{
    api::{result_audit_err, validate, ApiResult, ValidateRequest, ValidateRequestQuery},
    Audit, AuditBuilder, AuditListFilter, AuditListQuery, AuditMeta, AuditRead, AuditType,
    AuditUpdate, Driver, DEFAULT_LIMIT,
};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct AuditListRequest {
    #[builder(default = "None")]
    ge: Option<DateTime<Utc>>,

    #[builder(default = "None")]
    le: Option<DateTime<Utc>>,

    #[builder(default = "None")]
    #[validate(custom = "validate::limit")]
    limit: Option<i64>,

    #[builder(default = "None")]
    offset_id: Option<Uuid>,

    #[builder(default = "None")]
    id: Option<Vec<Uuid>>,

    #[builder(default = "None")]
    #[serde(rename = "type")]
    #[validate(custom = "validate::audit_type_vec")]
    type_: Option<Vec<String>>,

    #[builder(default = "None")]
    #[validate(custom = "validate::audit_subject_vec")]
    subject: Option<Vec<String>>,

    #[builder(default = "None")]
    service_id: Option<Vec<Uuid>>,

    #[builder(default = "None")]
    user_id: Option<Vec<Uuid>>,
}

impl ValidateRequest<AuditListRequest> for AuditListRequest {}
impl ValidateRequestQuery<AuditListRequest> for AuditListRequest {}

impl AuditListRequest {
    pub fn into_query_filter(self) -> (AuditListQuery, AuditListFilter) {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        let query = match (self.ge, self.le) {
            (Some(ge), Some(le)) => AuditListQuery::CreatedLeAndGe(le, ge, limit, self.offset_id),
            (Some(ge), None) => AuditListQuery::CreatedGe(ge, limit, self.offset_id),
            (None, Some(le)) => AuditListQuery::CreatedLe(le, limit, self.offset_id),
            (None, None) => AuditListQuery::CreatedLe(Utc::now(), limit, self.offset_id),
        };

        let filter = AuditListFilter {
            id: self.id,
            type_: self.type_,
            subject: self.subject,
            service_id: self.service_id,
            user_id: self.user_id,
        };

        (query, filter)
    }

    pub fn from_query_filter(query: AuditListQuery, filter: AuditListFilter) -> Self {
        match query {
            AuditListQuery::CreatedLe(le, limit, offset_id) => Self {
                ge: None,
                le: Some(le),
                limit: Some(limit),
                offset_id,
                id: filter.id,
                type_: filter.type_,
                subject: filter.subject,
                service_id: filter.service_id,
                user_id: filter.user_id,
            },
            AuditListQuery::CreatedGe(ge, limit, offset_id) => Self {
                ge: Some(ge),
                le: None,
                limit: Some(limit),
                offset_id,
                id: filter.id,
                type_: filter.type_,
                subject: filter.subject,
                service_id: filter.service_id,
                user_id: filter.user_id,
            },
            AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id) => Self {
                ge: Some(ge),
                le: Some(le),
                limit: Some(limit),
                offset_id,
                id: filter.id,
                type_: filter.type_,
                subject: filter.subject,
                service_id: filter.service_id,
                user_id: filter.user_id,
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

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct AuditCreateRequest {
    #[serde(rename = "type")]
    #[validate(custom = "validate::audit_type")]
    pub type_: String,

    #[builder(default = "None")]
    #[validate(custom = "validate::audit_subject")]
    pub subject: Option<String>,

    #[builder(default = "None")]
    pub data: Option<Value>,

    #[builder(default = "None")]
    pub user_id: Option<Uuid>,

    #[builder(default = "None")]
    pub user_key_id: Option<Uuid>,
}

impl ValidateRequest<AuditCreateRequest> for AuditCreateRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditReadRequest {
    #[validate(custom = "validate::audit_subject")]
    pub subject: Option<String>,
}

impl ValidateRequest<AuditReadRequest> for AuditReadRequest {}
impl ValidateRequestQuery<AuditReadRequest> for AuditReadRequest {}

impl AuditReadRequest {
    pub fn new(subject: Option<String>) -> Self {
        Self { subject }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditReadResponse {
    pub data: Audit,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditIdOptResponse {
    pub audit: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuditUpdateRequest {
    pub status_code: Option<u16>,

    #[validate(custom = "validate::audit_subject")]
    pub subject: Option<String>,

    pub data: Option<Value>,
}

impl Default for AuditUpdateRequest {
    fn default() -> Self {
        Self {
            status_code: None,
            subject: None,
            data: None,
        }
    }
}

impl AuditUpdateRequest {
    pub fn status_code(mut self, status_code: u16) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn data<D: Serialize>(mut self, data: D) -> Self {
        self.data = Some(serde_json::to_value(data).unwrap());
        self
    }

    pub fn subject<S: Into<String>>(mut self, subject: S) -> Self {
        self.subject = Some(subject.into());
        self
    }
}

impl From<AuditUpdateRequest> for AuditUpdate {
    fn from(request: AuditUpdateRequest) -> Self {
        Self {
            status_code: request.status_code,
            subject: request.subject,
            data: request.data,
        }
    }
}

pub fn audit_list(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuditListRequest,
) -> ApiResult<AuditListResponse> {
    AuditListRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditList);
    let (query, filter) = request.into_query_filter();

    let res = server_audit::list(driver, &mut audit, key_value, &query, &filter);
    result_audit_err(driver, &audit, res).map(|data| AuditListResponse {
        meta: AuditListRequest::from_query_filter(query, filter),
        data,
    })
}

pub fn audit_create(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: AuditCreateRequest,
) -> ApiResult<AuditReadResponse> {
    AuditCreateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditCreate);

    let res = server_audit::create(driver, &mut audit, key_value, request);
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}

pub fn audit_read(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    audit_id: Uuid,
    request: AuditReadRequest,
) -> ApiResult<AuditReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditRead);

    let res = server_audit::read(driver, &mut audit, key_value, audit_id, request);
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}

pub fn audit_update(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    audit_id: Uuid,
    request: AuditUpdateRequest,
) -> ApiResult<AuditReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditUpdate);
    let update: AuditUpdate = request.into();

    let res = server_audit::update(driver, &mut audit, key_value, audit_id, update);
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}

mod server_audit {
    use super::*;
    use crate::{
        api::{ApiError, ApiResult},
        pattern::*,
        Audit, AuditBuilder, AuditList, AuditListFilter, AuditListQuery, Driver, DriverError,
    };

    pub fn list(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        query: &AuditListQuery,
        filter: &AuditListFilter,
    ) -> ApiResult<Vec<Audit>> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let list = AuditList {
            query,
            filter,
            service_id: service.map(|s| s.id),
        };
        driver.audit_list(&list).map_err(ApiError::BadRequest)
    }

    pub fn create(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        request: AuditCreateRequest,
    ) -> ApiResult<Audit> {
        let _service =
            key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        audit
            .user_id(request.user_id)
            .user_key_id(request.user_key_id)
            .create(driver, request.type_, request.subject, request.data)
            .map_err(ApiError::BadRequest)
    }

    pub fn read(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        audit_id: Uuid,
        request: AuditReadRequest,
    ) -> ApiResult<Audit> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        driver
            .audit_read(
                &AuditRead::new(audit_id)
                    .subject(request.subject)
                    .service_id(service.map(|x| x.id)),
            )
            .map_err(ApiError::BadRequest)?
            .ok_or_else(|| ApiError::NotFound(DriverError::AuditNotFound))
    }

    pub fn update(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        audit_id: Uuid,
        update: AuditUpdate,
    ) -> ApiResult<Audit> {
        let service = key_authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        driver
            .audit_update(&audit_id, &update, service.map(|x| x.id))
            .map_err(ApiError::BadRequest)
    }
}
