//! # API functions.
mod auth;
mod error;
mod password;
pub mod validate;

pub use crate::api::{
    auth::*,
    error::*,
    password::*,
    validate::{ValidateRequest, ValidateRequestQuery},
};

use crate::{
    AuditBuilder, AuditDiff, AuditDiffBuilder, AuditSubject, Driver, DriverError, Service,
};
use http::StatusCode;
use tonic::Status;
use uuid::Uuid;

// TODO(refactor): Unwrap check and cleanup.

fn result_audit<T>(driver: &dyn Driver, audit: &AuditBuilder, res: ApiResult<T>) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", StatusData::from_status(&e));
        audit
            .create_data(driver, e.code() as u16, None, Some(data))
            .unwrap();
        Err(e)
    })
    .and_then(|res| {
        audit
            .create_data::<bool>(driver, StatusCode::OK.as_u16(), None, None)
            .unwrap();
        Ok(res)
    })
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditIdOptResponse {
    pub audit: Option<Uuid>,
}

pub fn result_audit_err<T>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: ApiResult<T>,
) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", StatusData::from_status(&e));
        audit
            .create_data(driver, e.code() as u16, None, Some(data))
            .unwrap();
        Err(e)
    })
}

pub fn result_audit_subject<T: AuditSubject>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: ApiResult<T>,
) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", StatusData::from_status(&e));
        audit
            .create_data(driver, e.code() as u16, None, Some(data))
            .unwrap();
        Err(e)
    })
    .and_then(|res| {
        audit
            .create_data::<bool>(driver, StatusCode::OK.as_u16(), Some(res.subject()), None)
            .unwrap();
        Ok(res)
    })
}

pub fn result_audit_diff<T: AuditSubject + AuditDiff>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: ApiResult<(T, T)>,
) -> ApiResult<T> {
    res.or_else(|e| {
        let data = AuditDiffBuilder::typed_data("error", StatusData::from_status(&e));
        audit
            .create_data(driver, e.code() as u16, None, Some(data))
            .unwrap();
        Err(e)
    })
    .and_then(|(p, n)| {
        let diff = n.diff(&p);
        audit
            .create_data(
                driver,
                StatusCode::OK.as_u16(),
                Some(n.subject()),
                Some(diff),
            )
            .unwrap();
        Ok(n)
    })
}

fn csrf_verify(driver: &dyn Driver, service: &Service, csrf_key: &str) -> ApiResult<()> {
    driver
        .csrf_read(&csrf_key)
        .map_err(ApiError::BadRequest)
        .map_err::<Status, _>(Into::into)?
        .ok_or_else(|| DriverError::CsrfNotFoundOrUsed)
        .and_then(|csrf| csrf.check_service(service.id))
        .map_err(ApiError::BadRequest)
        .map_err::<Status, _>(Into::into)
}
