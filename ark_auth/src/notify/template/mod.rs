use crate::notify::{NotifyActor, NotifyError};
use handlebars::Handlebars;
use serde_json::Value;

const EMAIL_RESET_PASSWORD: &str = "email_reset_password";
const EMAIL_UPDATE_EMAIL: &str = "email_update_email";
const EMAIL_UPDATE_PASSWORD: &str = "email_update_password";

impl NotifyActor {
    /// Register default template strings.
    pub fn template_register_default(registry: &mut Handlebars) -> Result<(), NotifyError> {
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
    pub fn template_email_reset_password(&self, parameters: &Value) -> Result<String, NotifyError> {
        self.registry()
            .render(EMAIL_RESET_PASSWORD, parameters)
            .map_err(NotifyError::HandlebarsRender)
    }

    /// Render update email email template.
    pub fn template_email_update_email(&self, parameters: &Value) -> Result<String, NotifyError> {
        self.registry()
            .render(EMAIL_UPDATE_EMAIL, parameters)
            .map_err(NotifyError::HandlebarsRender)
    }

    /// Render update password email template.
    pub fn template_email_update_password(
        &self,
        parameters: &Value,
    ) -> Result<String, NotifyError> {
        self.registry()
            .render(EMAIL_UPDATE_PASSWORD, parameters)
            .map_err(NotifyError::HandlebarsRender)
    }
}
