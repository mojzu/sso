use crate::{
    AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES, NAME_MAX_LEN, USER_LOCALE_MAX_LEN,
    USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN, USER_TIMEZONE_MAX_LEN,
};
use tonic::Status;
use validator::{Validate, ValidationError, ValidationErrors};

pub fn email(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if !validator::validate_email(value) {
        errors.add(field, ValidationError::new("email_invalid"));
    }
}

pub fn password(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.len() < USER_PASSWORD_MIN_LEN || value.len() > USER_PASSWORD_MAX_LEN {
        errors.add(field, ValidationError::new("password_invalid"));
    }
}

pub fn password_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        password(errors, field, value);
    }
}

pub fn name(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > NAME_MAX_LEN {
        errors.add(field, ValidationError::new("name_invalid"));
    }
}

pub fn locale(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    use unic_langid::LanguageIdentifier;

    if let Err(_e) = value.parse::<LanguageIdentifier>() {
        errors.add(field, ValidationError::new("locale_invalid"));
    } else if value.is_empty() || value.len() > USER_LOCALE_MAX_LEN {
        errors.add(field, ValidationError::new("locale_invalid"));
    }
}

pub fn locale_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        locale(errors, field, value);
    }
}

pub fn timezone(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    use chrono_tz::Tz;
    use std::str::FromStr;

    if let Err(_e) = Tz::from_str(value) {
        errors.add(field, ValidationError::new("timezone_invalid"));
    } else if value.is_empty() || value.len() > USER_TIMEZONE_MAX_LEN {
        errors.add(field, ValidationError::new("timezone_invalid"));
    }
}

pub fn timezone_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        timezone(errors, field, value);
    }
}

pub fn token(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > JWT_MAX_LEN {
        errors.add(field, ValidationError::new("token_invalid"));
    }
}

pub fn key(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > (KEY_VALUE_BYTES * 2) {
        errors.add(field, ValidationError::new("key_invalid"));
    }
}

pub fn audit_type(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > AUDIT_TYPE_MAX_LEN {
        errors.add(field, ValidationError::new("audit_type_invalid"));
    }
}

pub fn audit_type_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        audit_type(errors, field, value);
    }
}

pub fn wrap<F>(f: F) -> Result<(), ValidationErrors>
where
    F: FnOnce(&mut ValidationErrors),
{
    let mut errors = ValidationErrors::new();
    f(&mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validate<T>(x: T) -> Result<T, Status>
where
    T: Validate,
{
    x.validate()
        .map_err(|e| Status::invalid_argument(format!("{}", e)))?;
    Ok(x)
}
