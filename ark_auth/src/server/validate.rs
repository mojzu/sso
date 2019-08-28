use crate::server::Error;
use futures::future;
use serde::de::DeserializeOwned;
use serde_json::Value;
use validator::{Validate, ValidationError};

/// Validate JSON value trait.
pub trait FromJsonValue<T: DeserializeOwned + Validate> {
    /// Extract and validate data from JSON value.
    fn from_value(value: Value) -> future::FutureResult<T, Error> {
        future::result(
            serde_json::from_value::<T>(value)
                .map_err(|_err| Error::BadRequest)
                .and_then(|body| {
                    body.validate().map_err(|_err| Error::BadRequest)?;
                    Ok(body)
                }),
        )
    }
}

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

pub fn offset(offset: &str) -> Result<(), ValidationError> {
    offset
        .parse::<bool>()
        .map_err(|_err| ValidationError::new("invalid_offset"))?;
    Ok(())
}

pub fn password(password: &str) -> Result<(), ValidationError> {
    if password.is_empty() || password.len() > 100 {
        Err(ValidationError::new("invalid_password"))
    } else {
        Ok(())
    }
}

pub fn name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 100 {
        Err(ValidationError::new("invalid_name"))
    } else {
        Ok(())
    }
}

pub fn path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() || path.len() > 200 {
        Err(ValidationError::new("invalid_path"))
    } else {
        Ok(())
    }
}

pub fn token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() || token.len() > 1000 {
        Err(ValidationError::new("invalid_token"))
    } else {
        Ok(())
    }
}

pub fn key(key: &str) -> Result<(), ValidationError> {
    if key.is_empty() || key.len() > 32 {
        Err(ValidationError::new("invalid_key"))
    } else {
        Ok(())
    }
}
