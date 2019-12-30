//! # API Validation
use crate::{
    api::{ApiError, ApiResult},
    DriverError, AUDIT_SUBJECT_MAX_LEN, AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES,
    NAME_MAX_LEN, TEXT_MAX_LEN, USER_LOCALE_MAX_LEN, USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN,
    USER_TIMEZONE_MAX_LEN,
};
use chrono_tz::Tz;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use tonic::Status;
use unic_langid::LanguageIdentifier;
use validator::{validate_email, Validate, ValidationError};

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

/// API deserialise from request query string trait.
pub trait ValidateRequestQuery<T: DeserializeOwned> {
    fn from_str(s: &str) -> ApiResult<T> {
        serde_qs::from_str(s)
            .map_err::<DriverError, _>(Into::into)
            .map_err(ApiError::BadRequest)
            .map_err::<tonic::Status, _>(Into::into)
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

pub fn email(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() || !validate_email(value) {
        Err(ValidationError::new("invalid_email"))
    } else {
        Ok(())
    }
}

pub fn email_vec(value: &[String]) -> Result<(), ValidationError> {
    for v in value {
        email(v)?;
    }
    Ok(())
}

pub fn locale(locale: &str) -> Result<(), ValidationError> {
    if let Err(_e) = locale.parse::<LanguageIdentifier>() {
        Err(ValidationError::new("invalid_locale"))
    } else if locale.is_empty() || locale.len() > USER_LOCALE_MAX_LEN {
        Err(ValidationError::new("invalid_locale"))
    } else {
        Ok(())
    }
}

pub fn timezone(timezone: &str) -> Result<(), ValidationError> {
    if let Err(_e) = Tz::from_str(timezone) {
        Err(ValidationError::new("invalid_timezone"))
    } else if timezone.is_empty() || timezone.len() > USER_TIMEZONE_MAX_LEN {
        Err(ValidationError::new("invalid_timezone"))
    } else {
        Ok(())
    }
}

pub fn password(password: &str) -> Result<(), ValidationError> {
    if password.len() < USER_PASSWORD_MIN_LEN || password.len() > USER_PASSWORD_MAX_LEN {
        Err(ValidationError::new("invalid_password"))
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
