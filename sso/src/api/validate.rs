//! # API Validation
use crate::{
    Core, CoreError, CoreResult, AUDIT_SUBJECT_MAX_LEN, AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN,
    KEY_VALUE_BYTES, USER_LOCALE_MAX_LEN, USER_NAME_MAX_LEN, USER_PASSWORD_MAX_LEN,
    USER_PASSWORD_MIN_LEN, USER_TIMEZONE_MAX_LEN,
};
use chrono_tz::Tz;
use futures::future;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;
use validator::{Validate, ValidationError};

/// API validate request trait.
pub trait ValidateRequest<T: Validate> {
    fn api_validate(t: &T) -> Result<(), CoreError> {
        t.validate().map_err(|e| {
            debug!("{}", e);
            CoreError::BadRequest
        })?;
        Ok(())
    }

    fn api_validate_fut(t: &T) -> future::FutureResult<(), CoreError> {
        future::result(Self::api_validate(t))
    }
}

/// API deserialise from request query string trait.
pub trait ValidateRequestQuery<T: DeserializeOwned> {
    fn from_str(s: &str) -> CoreResult<T> {
        Core::qs_de::<T>(s).map_err(|e| {
            debug!("{}", e);
            CoreError::BadRequest
        })
    }

    fn from_str_fut(s: &str) -> future::FutureResult<T, CoreError> {
        future::result(Self::from_str(s))
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
    if name.is_empty() || name.len() > USER_NAME_MAX_LEN {
        Err(ValidationError::new("invalid_name"))
    } else {
        Ok(())
    }
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
