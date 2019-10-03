use crate::{
    CoreError, CoreResult, CoreUtil, AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES,
    USER_LOCALE_MAX_LEN, USER_NAME_MAX_LEN, USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN,
    USER_TIMEZONE_MAX_LEN,
};
use chrono_tz::Tz;
use futures::future;
use serde::de::DeserializeOwned;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;
use validator::{Validate, ValidationError};

/// API validate request trait.
pub trait ApiValidateRequest<T: Validate> {
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
pub trait ApiValidateRequestQuery<T: DeserializeOwned> {
    fn from_str(s: &str) -> CoreResult<T> {
        CoreUtil::qs_de::<T>(s).map_err(|e| {
            debug!("{}", e);
            CoreError::BadRequest
        })
    }

    fn from_str_fut(s: &str) -> future::FutureResult<T, CoreError> {
        future::result(Self::from_str(s))
    }
}

/// API validation functions.
pub struct ApiValidate;

impl ApiValidate {
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

    pub fn audit_type(audit_type: &str) -> Result<(), ValidationError> {
        if audit_type.is_empty() || audit_type.len() > AUDIT_TYPE_MAX_LEN {
            Err(ValidationError::new("invalid_audit_type"))
        } else {
            Ok(())
        }
    }

    pub fn audit_type_vec(audit_type: &[String]) -> Result<(), ValidationError> {
        for v in audit_type {
            Self::audit_type(v)?;
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
}
