use crate::{
    grpc::util::{MethodError, MethodResult},
    DriverError, KeyType, AUDIT_SUBJECT_MAX_LEN, AUDIT_TYPE_MAX_LEN, JWT_MAX_LEN, KEY_VALUE_BYTES,
    NAME_MAX_LEN, TEXT_MAX_LEN, TOTP_MAX_LEN, USER_LOCALE_MAX_LEN, USER_PASSWORD_MAX_LEN,
    USER_PASSWORD_MIN_LEN, USER_TIMEZONE_MAX_LEN,
};
use std::convert::TryInto;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

pub fn email(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if !validator::validate_email(value) {
        errors.add(field, ValidationError::new("email_invalid"));
    }
}

pub fn email_vec(errors: &mut ValidationErrors, field: &'static str, value: &[String]) {
    for v in value {
        email(errors, field, v);
    }
}

pub fn url(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if !validator::validate_url(value) {
        errors.add(field, ValidationError::new("url_invalid"));
    }
}

pub fn url_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        url(errors, field, value);
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

pub fn name_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        name(errors, field, value);
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

pub fn audit_type_vec(errors: &mut ValidationErrors, field: &'static str, value: &[String]) {
    for v in value {
        audit_type(errors, field, v);
    }
}

pub fn limit(errors: &mut ValidationErrors, field: &'static str, value: i64) {
    if value < 0 {
        errors.add(field, ValidationError::new("limit_invalid"));
    }
}

pub fn limit_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<i64>) {
    if let Some(value) = value {
        limit(errors, field, value);
    }
}

pub fn uuid(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if let Err(_e) = Uuid::parse_str(value) {
        errors.add(field, ValidationError::new("uuid_invalid"));
    }
}

pub fn uuid_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        uuid(errors, field, value);
    }
}

pub fn uuid_vec(errors: &mut ValidationErrors, field: &'static str, value: &[String]) {
    for v in value {
        uuid(errors, field, v);
    }
}

pub fn audit_subject(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > AUDIT_SUBJECT_MAX_LEN {
        errors.add(field, ValidationError::new("audit_subject_invalid"));
    }
}

pub fn audit_subject_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        audit_subject(errors, field, value);
    }
}

pub fn audit_subject_vec(errors: &mut ValidationErrors, field: &'static str, value: &[String]) {
    for v in value {
        audit_subject(errors, field, v);
    }
}

pub fn key_type(errors: &mut ValidationErrors, field: &'static str, value: i32) {
    let x: Result<KeyType, ()> = value.try_into();
    if let Err(_e) = x {
        errors.add(field, ValidationError::new("key_type_invalid"));
    }
}

pub fn key_type_vec(errors: &mut ValidationErrors, field: &'static str, value: &[i32]) {
    for v in value {
        key_type(errors, field, *v);
    }
}

pub fn text(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > TEXT_MAX_LEN {
        errors.add(field, ValidationError::new("text_invalid"));
    }
}

pub fn text_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<&str>) {
    if let Some(value) = value {
        text(errors, field, value);
    }
}

pub fn totp(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    if value.is_empty() || value.len() > TOTP_MAX_LEN {
        errors.add(field, ValidationError::new("totp_invalid"));
    }
}

pub fn csrf_token(errors: &mut ValidationErrors, field: &'static str, value: &str) {
    key(errors, field, value);
}

pub fn csrf_expires_s(errors: &mut ValidationErrors, field: &'static str, value: i64) {
    if value < 0 || value > 86400 {
        errors.add(field, ValidationError::new("csrf_expires_s_invalid"));
    }
}

pub fn csrf_expires_s_opt(errors: &mut ValidationErrors, field: &'static str, value: Option<i64>) {
    if let Some(value) = value {
        csrf_expires_s(errors, field, value);
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

pub fn validate<T>(x: T) -> MethodResult<T>
where
    T: Validate,
{
    x.validate()
        .map_err(|e| MethodError::BadRequest(DriverError::Validation(e)))?;
    Ok(x)
}
