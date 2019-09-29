use crate::{
    CoreUtil, ServerError, AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES, USER_NAME_MAX_LEN,
    USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN,
};
use futures::future;
use serde::de::DeserializeOwned;
use serde_json::Value;
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

    pub fn key(key: &str) -> Result<(), ValidationError> {
        if key.len() != (KEY_VALUE_BYTES * 2) {
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
