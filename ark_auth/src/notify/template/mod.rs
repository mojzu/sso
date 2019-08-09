use crate::notify::NotifyError;
use handlebars::Handlebars;
use serde_json::Value;

const EMAIL_RESET_PASSWORD: &str = "email_reset_password";
const EMAIL_UPDATE_EMAIL: &str = "email_update_email";
const EMAIL_UPDATE_PASSWORD: &str = "email_update_password";

/// Register template strings.
pub fn register(registry: &mut Handlebars) -> Result<(), NotifyError> {
    registry
        .register_template_string(
            EMAIL_RESET_PASSWORD,
            include_str!("email_reset_password.hbs"),
        )
        .map_err(NotifyError::HandlebarsTemplate)?;
    registry
        .register_template_string(EMAIL_UPDATE_EMAIL, include_str!("email_update_email.hbs"))
        .map_err(NotifyError::HandlebarsTemplate)?;
    registry
        .register_template_string(
            EMAIL_UPDATE_PASSWORD,
            include_str!("email_update_password.hbs"),
        )
        .map_err(NotifyError::HandlebarsTemplate)?;
    Ok(())
}

/// Render reset password email template.
pub fn email_reset_password(
    registry: &Handlebars,
    parameters: &Value,
) -> Result<String, NotifyError> {
    registry
        .render(EMAIL_RESET_PASSWORD, parameters)
        .map_err(NotifyError::HandlebarsRender)
}

/// Render update email email template.
pub fn email_update_email(
    registry: &Handlebars,
    parameters: &Value,
) -> Result<String, NotifyError> {
    registry
        .render(EMAIL_UPDATE_EMAIL, parameters)
        .map_err(NotifyError::HandlebarsRender)
}

/// Render update password email template.
pub fn email_update_password(
    registry: &Handlebars,
    parameters: &Value,
) -> Result<String, NotifyError> {
    registry
        .render(EMAIL_UPDATE_PASSWORD, parameters)
        .map_err(NotifyError::HandlebarsRender)
}
