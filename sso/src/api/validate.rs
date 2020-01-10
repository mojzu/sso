//! # API Validation
use crate::{
    api::{ApiError, ApiResult},
    DriverError, AUDIT_SUBJECT_MAX_LEN, AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES,
    NAME_MAX_LEN, TEXT_MAX_LEN,
};
use tonic::Status;
use validator::{Validate, ValidationError};

/// API validate request trait.
pub trait ValidateRequest<T: Validate> {
    fn api_validate(t: &T) -> ApiResult<()> {
        t.validate()
            .map_err::<DriverError, _>(Into::into)
            .map_err(ApiError::BadRequest)
            .map_err::<tonic::Status, _>(Into::into)
    }

    fn status_validate(t: &T) -> Result<(), Status> {
        t.validate()
            .map_err(|e| Status::invalid_argument(format!("{}", e)))
    }
}

pub fn limit(limit: i64) -> Result<(), ValidationError> {
    if limit < 0 {
        Err(ValidationError::new("invalid_limit"))
    } else {
        Ok(())
    }
}

pub fn name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > NAME_MAX_LEN {
        Err(ValidationError::new("invalid_name"))
    } else {
        Ok(())
    }
}

pub fn text(text: &str) -> Result<(), ValidationError> {
    if text.is_empty() || text.len() > TEXT_MAX_LEN {
        Err(ValidationError::new("invalid_text"))
    } else {
        Ok(())
    }
}

pub fn audit_type(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > AUDIT_TYPE_MAX_LEN {
        Err(ValidationError::new("invalid_audit_type"))
    } else {
        Ok(())
    }
}

pub fn audit_type_vec(value: &[String]) -> Result<(), ValidationError> {
    for v in value {
        audit_type(v)?;
    }
    Ok(())
}

pub fn audit_subject(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || value.len() > AUDIT_SUBJECT_MAX_LEN {
        Err(ValidationError::new("invalid_audit_subject"))
    } else {
        Ok(())
    }
}

pub fn audit_subject_vec(value: &[String]) -> Result<(), ValidationError> {
    for v in value {
        audit_subject(v)?;
    }
    Ok(())
}

pub fn token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() || token.len() > JWT_MAX_LEN {
        Err(ValidationError::new("invalid_token"))
    } else {
        Ok(())
    }
}

pub fn totp(totp: &str) -> Result<(), ValidationError> {
    if totp.is_empty() || totp.len() > 10 {
        Err(ValidationError::new("invalid_totp"))
    } else {
        Ok(())
    }
}

pub fn key(key: &str) -> Result<(), ValidationError> {
    if key.is_empty() || key.len() > (KEY_VALUE_BYTES * 2) {
        Err(ValidationError::new("invalid_key"))
    } else {
        Ok(())
    }
}

pub fn csrf_key(csrf_key: &str) -> Result<(), ValidationError> {
    key(csrf_key)
}

pub fn csrf_expires_s(expires_s: i64) -> Result<(), ValidationError> {
    if expires_s < 0 || expires_s > 86400 {
        Err(ValidationError::new("invalid_expires_s"))
    } else {
        Ok(())
    }
}
