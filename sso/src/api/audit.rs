use crate::{
    api::{result_audit_err, validate, ValidateRequest, ValidateRequestQuery},
    Audit, AuditBuilder, AuditCreate2, AuditListFilter, AuditListQuery, AuditMeta, AuditType,
    AuditUpdate, Core, CoreResult, Driver, Key,
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
        let limit = self.limit.unwrap_or_else(Core::default_limit);
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

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct AuditCreate2Request {
    #[serde(rename = "type")]
    #[validate(custom = "validate::audit_type")]
    pub type_: String,
    #[builder(default = "None")]
    #[validate(custom = "validate::audit_subject")]
    pub subject: Option<String>,
    #[builder(default = "None")]
    pub data: Option<Value>,
}

impl ValidateRequest<AuditCreate2Request> for AuditCreate2Request {}

impl From<AuditCreate2Request> for AuditCreate2 {
    fn from(data: AuditCreate2Request) -> Self {
        Self {
            type_: data.type_,
            subject: data.subject,
            data: data.data,
        }
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
    #[validate(custom = "validate::audit_subject")]
    pub subject: Option<String>,
    pub data: Option<Value>,
}

impl Default for AuditUpdateRequest {
    fn default() -> Self {
        Self {
            subject: None,
            data: None,
        }
    }
}

impl AuditUpdateRequest {
    pub fn data<S: Serialize>(mut self, data: S) -> Self {
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
            subject: request.subject,
            data: request.data,
        }
    }
}

pub fn audit_list(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuditListRequest,
) -> CoreResult<AuditListResponse> {
    AuditListRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditList);
    let (query, filter) = request.into_query_filter();

    let res = Key::authenticate(driver, &mut audit, key_value)
        .and_then(|service| Audit::list(driver, service.as_ref(), &query, &filter));
    result_audit_err(driver, &audit, res).map(|data| AuditListResponse {
        meta: AuditListRequest::from_query_filter(query, filter),
        data,
    })
}

pub fn audit_create(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: AuditCreateRequest,
) -> CoreResult<AuditReadResponse> {
    AuditCreateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditCreate);
    let audit_create = AuditCreate2::new(request.type_, request.subject, request.data);
    let user_id = request.user_id;
    let user_key_id = request.user_key_id;

    let res = Key::authenticate(driver, &mut audit, key_value).and_then(|_service| {
        audit
            .user_id(user_id)
            .user_key_id(user_key_id)
            .create(driver, audit_create)
    });
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}

pub fn audit_read(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    audit_id: Uuid,
) -> CoreResult<AuditReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditRead);

    let res = Key::authenticate(driver, &mut audit, key_value)
        .and_then(|service| Audit::read(driver, service.as_ref(), &audit_id));
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}

pub fn audit_update(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    audit_id: Uuid,
    request: AuditUpdateRequest,
) -> CoreResult<AuditReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditUpdate);
    let update: AuditUpdate = request.into();

    let res = Key::authenticate(driver, &mut audit, key_value)
        .and_then(|service| Audit::update(driver, service.as_ref(), &audit_id, &update));
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}
