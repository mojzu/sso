use crate::{
    CoreUtil, ServerError, ServerResult, JWT_MAX_LEN, KEY_VALUE_BYTES, USER_LOCALE_MAX_LEN,
    USER_NAME_MAX_LEN, USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN, USER_TIMEZONE_MAX_LEN,
};
use chrono_tz::Tz;
use futures::future;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::str::FromStr;
use unic_langid::LanguageIdentifier;
use validator::{Validate, ValidationError};

/// Server validate JSON value trait.
pub trait ServerValidateFromValue<T: DeserializeOwned + Validate> {
    /// Extract and validate data from JSON value.
    fn from_value(value: Value) -> future::FutureResult<T, ServerError> {
        future::result(
            serde_json::from_value::<T>(value)
                .map_err(|_e| {
                    debug!("{}", _e);
                    ServerError::BadRequest
                })
                .and_then(|body| {
                    body.validate().map_err(|_e| {
                        debug!("{}", _e);
                        ServerError::BadRequest
                    })?;
                    Ok(body)
                }),
        )
    }
}

/// Server validate query string value trait.
pub trait ServerValidateFromStr<T: DeserializeOwned + Validate> {
    /// Extract and validate data from query string value.
    fn from_str(value: &str) -> future::FutureResult<T, ServerError> {
        future::result(
            CoreUtil::qs_de::<T>(value)
                .map_err(|_e| {
                    debug!("{}", _e);
                    ServerError::BadRequest
                })
                .and_then(|body| {
                    body.validate().map_err(|_e| {
                        debug!("{}", _e);
                        ServerError::BadRequest
                    })?;
                    Ok(body)
                }),
        )
    }
}

/// Server validation functions.
pub struct ServerValidate;

impl ServerValidate {
    pub fn qs_de<T: DeserializeOwned>(v: &str) -> ServerResult<T> {
        CoreUtil::qs_de::<T>(v).map_err(|_e| {
            debug!("{}", _e);
            ServerError::BadRequest
        })
    }

    pub fn limit(limit: i64) -> Result<(), ValidationError> {
        if limit < 0 {
            Err(ValidationError::new("invalid_limit"))
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

    pub fn token(token: &str) -> Result<(), ValidationError> {
        if token.is_empty() || token.len() > JWT_MAX_LEN {
            Err(ValidationError::new("invalid_token"))
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

    pub fn totp(totp: &str) -> Result<(), ValidationError> {
        if totp.is_empty() || totp.len() > 10 {
            Err(ValidationError::new("invalid_totp"))
        } else {
            Ok(())
        }
    }
}
