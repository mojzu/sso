use crate::{
    ServerError, AUDIT_PATH_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES, USER_NAME_MAX_LEN,
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
                .map_err(|_err| ServerError::BadRequest)
                .and_then(|body| {
                    body.validate().map_err(|_err| ServerError::BadRequest)?;
                    Ok(body)
                }),
        )
    }
}

/// Server validation functions.
pub struct ServerValidate;

impl ServerValidate {
    pub fn limit(limit: &str) -> Result<(), ValidationError> {
        let limit = limit
            .parse::<i64>()
            .map_err(|_err| ValidationError::new("invalid_limit"))?;
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

    pub fn path(path: &str) -> Result<(), ValidationError> {
        if path.is_empty() || path.len() > AUDIT_PATH_MAX_LEN {
            Err(ValidationError::new("invalid_path"))
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
        if key.len() != (KEY_VALUE_BYTES * 2) {
            Err(ValidationError::new("invalid_key"))
        } else {
            Ok(())
        }
    }
}
