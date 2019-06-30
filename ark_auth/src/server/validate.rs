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

pub fn limit(limit: i64) -> Result<(), ValidationError> {
    if limit < 0 {
        Err(ValidationError::new("invalid_limit"))
    } else {
        Ok(())
    }
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

pub fn email_subject(email_subject: &str) -> Result<(), ValidationError> {
    if email_subject.is_empty() || email_subject.len() > 200 {
        Err(ValidationError::new("invalid_email_subject"))
    } else {
        Ok(())
    }
}

pub fn email_text(email_text: &str) -> Result<(), ValidationError> {
    if email_text.is_empty() || email_text.len() > 1000 {
        Err(ValidationError::new("invalid_email_text"))
    } else {
        Ok(())
    }
}

pub fn email_link_text(email_link_text: &str) -> Result<(), ValidationError> {
    if email_link_text.is_empty() || email_link_text.len() > 200 {
        Err(ValidationError::new("invalid_email_link_text"))
    } else {
        Ok(())
    }
}

pub fn id(id: &str) -> Result<(), ValidationError> {
    if id.is_empty() || id.len() > 32 {
        Err(ValidationError::new("invalid_id"))
    } else {
        Ok(())
    }
}

pub fn token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() || token.len() > 1024 {
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
