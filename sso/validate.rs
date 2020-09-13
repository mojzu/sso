//! # Validate
use crate::internal::*;

/// Validates a vector of email address strings
pub fn email_vec(values: &Vec<String>) -> std::result::Result<(), ValidationError> {
    for value in values {
        if !validator::validate_email(value) {
            return Err(ValidationError::new("email_invalid"));
        }
    }
    Ok(())
}

/// Validates a database ID
pub fn id(value: i64) -> std::result::Result<(), ValidationError> {
    if value < 1 {
        return Err(ValidationError::new("id_invalid"));
    }
    Ok(())
}

/// Validates a vector of database IDs
pub fn id_vec(values: &Vec<i64>) -> std::result::Result<(), ValidationError> {
    for value in values {
        id(*value)?;
    }
    Ok(())
}

/// Validates an audit type
pub fn audit_type(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 100 {
        return Err(ValidationError::new("audit_type_invalid"));
    }
    Ok(())
}

/// Validates a vector of audit types
pub fn audit_type_vec(values: &Vec<String>) -> std::result::Result<(), ValidationError> {
    for value in values {
        audit_type(value)?;
    }
    Ok(())
}

/// Validates an audit subject
pub fn audit_subject(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 1000 {
        return Err(ValidationError::new("audit_subject_invalid"));
    }
    Ok(())
}

/// Validates a vector of audit subjects
pub fn audit_subject_vec(values: &Vec<String>) -> std::result::Result<(), ValidationError> {
    for value in values {
        audit_subject(value)?;
    }
    Ok(())
}

/// Validates a client ID
pub fn client_id(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 100 {
        return Err(ValidationError::new("client_id_invalid"));
    }
    Ok(())
}

/// Validates a CSRF token
pub fn csrf_token(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 500 {
        return Err(ValidationError::new("csrf_token_invalid"));
    }
    Ok(())
}

/// Validates a code
pub fn code(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 1000 {
        return Err(ValidationError::new("code_invalid"));
    }
    Ok(())
}

/// Validates a state
pub fn state(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 1000 {
        return Err(ValidationError::new("state_invalid"));
    }
    Ok(())
}

/// Validates a token
pub fn token(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 1000 {
        return Err(ValidationError::new("token_invalid"));
    }
    Ok(())
}

/// Validates a scope
pub fn scope(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() > 1000 {
        return Err(ValidationError::new("scope_invalid"));
    }
    Ok(())
}

/// Validates an OAuth2 provider
pub fn oauth2_provider(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < 1 || value.len() > 20 {
        return Err(ValidationError::new("oauth2_provider_invalid"));
    }
    Ok(())
}

/// Validates a locale
pub fn locale(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() > 100 {
        return Err(ValidationError::new("locale_invalid"));
    }
    Ok(())
}

/// Validates a timezone
pub fn timezone(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() > 500 {
        return Err(ValidationError::new("timezone_invalid"));
    }
    Ok(())
}

pub(crate) const PASSWORD_MIN: usize = 8;
pub(crate) const PASSWORD_MAX: usize = 64;

/// Validates a password
pub fn password(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < PASSWORD_MIN || value.len() > PASSWORD_MAX {
        return Err(ValidationError::new("password_invalid"));
    }
    Ok(())
}

pub(crate) const NAME_MIN: usize = 1;
pub(crate) const NAME_MAX: usize = 100;

/// Validates a name
pub fn name(value: &str) -> std::result::Result<(), ValidationError> {
    if value.len() < NAME_MIN || value.len() > NAME_MAX {
        return Err(ValidationError::new("name_invalid"));
    }
    Ok(())
}
