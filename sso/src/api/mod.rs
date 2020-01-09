//! # API functions.
mod error;
mod password;
pub mod validate;

pub use crate::api::{error::*, password::*, validate::ValidateRequest};

use crate::{AuditBuilder, AuditDiff, AuditDiffBuilder, AuditSubject, Driver};
use http::StatusCode;
use uuid::Uuid;

// TODO(refactor): Unwrap check and cleanup.

pub fn result_audit<T>(
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
