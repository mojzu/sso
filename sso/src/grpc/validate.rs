use crate::{USER_PASSWORD_MAX_LEN, USER_PASSWORD_MIN_LEN};
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

pub fn wrapper<F>(f: F) -> Result<(), ValidationErrors>
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
