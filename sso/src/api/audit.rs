use crate::{
    api::{result_audit_err, validate, ApiError, ApiResult, ValidateRequest, ValidateRequestQuery},
    pattern::*,
    Audit, AuditBuilder, AuditList, AuditListFilter, AuditListQuery, AuditMeta, AuditRead,
    AuditType, AuditUpdate, Driver, DriverError, DEFAULT_LIMIT,
};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use serde_json::Value;
use tonic::Status;
use uuid::Uuid;
use validator::Validate;

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

pub fn audit_read(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    audit_id: Uuid,
    request: AuditReadRequest,
) -> ApiResult<AuditReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditRead);

    fn read_inner(
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
            .map_err(ApiError::BadRequest)
            .map_err::<Status, _>(Into::into)?
            .ok_or_else(|| {
                let e: Status = ApiError::NotFound(DriverError::AuditNotFound).into();
                e
            })
    }

    let res = read_inner(driver, &mut audit, key_value, audit_id, request);
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

    fn update_inner(
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
            .map_err(Into::into)
    }

    let res = update_inner(driver, &mut audit, key_value, audit_id, update);
    result_audit_err(driver, &audit, res).map(|data| AuditReadResponse { data })
}
