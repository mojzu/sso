pub mod notify_msg;
mod smtp;

use crate::{AuditMeta, CoreError, DriverError};
use actix::{Actor, Addr, SyncArbiter, SyncContext};
use chrono::Utc;
use handlebars::Handlebars;
use serde_json::Value;
use std::fmt;

const EMAIL_RESET_PASSWORD: &str = "email_reset_password";
const EMAIL_UPDATE_EMAIL: &str = "email_update_email";
const EMAIL_UPDATE_PASSWORD: &str = "email_update_password";

/// Notify SMTP errors.
#[derive(Debug, Fail)]
pub enum NotifySmtpError {
    #[fail(display = "SmtpError:Disabled")]
    Disabled,

    #[fail(display = "SmtpError:NativeTls {}", _0)]
    NativeTls(#[fail(cause)] native_tls::Error),

    #[fail(display = "SmtpError:LettreEmail {}", _0)]
    LettreEmail(#[fail(cause)] lettre_email::error::Error),

    #[fail(display = "SmtpError:Lettre {}", _0)]
    Lettre(#[fail(cause)] lettre::smtp::error::Error),
}

/// Notify errors.
#[derive(Debug, Fail)]
pub enum NotifyError {
    #[fail(display = "NotifyError:Smtp {}", _0)]
    Smtp(#[fail(cause)] NotifySmtpError),

    #[fail(display = "SmtpError:Core {}", _0)]
    Core(#[fail(cause)] CoreError),

    #[fail(display = "SmtpError:Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

    #[fail(display = "NotifyError:HandlebarsTemplate {}", _0)]
    HandlebarsTemplate(#[fail(cause)] handlebars::TemplateError),

    #[fail(display = "NotifyError:HandlebarsRender {}", _0)]
    HandlebarsRender(#[fail(cause)] handlebars::RenderError),
}

/// Notify result wrapper type.
pub type NotifyResult<T> = Result<T, NotifyError>;

/// Notify actor SMTP options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyActorOptionsSmtp {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl NotifyActorOptionsSmtp {
    /// Create new SMTP options.
    pub fn new(host: String, port: u16, user: String, password: String) -> Self {
        Self {
            host,
            port,
            user,
            password,
        }
    }
}

/// Notify actor options.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NotifyActorOptions {
    #[builder(default)]
    smtp: Option<NotifyActorOptionsSmtp>,
}

/// Notify actor.
pub struct NotifyActor {
    options: NotifyActorOptions,
    registry: Handlebars,
}

impl fmt::Debug for NotifyActor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NotifyActor {{ options: {:?}, registry }}", self.options)
    }
}

impl NotifyActor {
    /// Start notifications actor on number of threads with options.
    pub fn start(threads: usize, options: NotifyActorOptions) -> Addr<Self> {
        SyncArbiter::start(threads, move || {
            // Register template strings.
            let mut handlebars = Handlebars::new();
            NotifyActor::template_register_default(&mut handlebars).unwrap();

            Self {
                options: options.clone(),
                registry: handlebars,
            }
        })
    }

    /// Returns template registry reference.
    pub fn registry(&self) -> &Handlebars {
        &self.registry
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

    /// Register default template strings.
    fn template_register_default(registry: &mut Handlebars) -> Result<(), NotifyError> {
        registry
            .register_template_string(
                EMAIL_RESET_PASSWORD,
                include_str!("template/email_reset_password.hbs"),
            )
            .map_err(NotifyError::HandlebarsTemplate)?;
        registry
            .register_template_string(
                EMAIL_UPDATE_EMAIL,
                include_str!("template/email_update_email.hbs"),
            )
            .map_err(NotifyError::HandlebarsTemplate)?;
        registry
            .register_template_string(
                EMAIL_UPDATE_PASSWORD,
                include_str!("template/email_update_password.hbs"),
            )
            .map_err(NotifyError::HandlebarsTemplate)?;
        Ok(())
    }

    /// Logs warning for error and returns ok.
    fn warn_on_err(err: NotifyError) -> Result<(), NotifyError> {
        warn!("{}", err);
        Ok(())
    }

    /// Returns optional audit log template values.
    fn audit_value(audit: &AuditMeta) -> Value {
        json!({
            "created_at": Utc::now(),
            "user_agent": audit.user_agent(),
            "remote": audit.remote(),
            "forwarded": audit.forwarded(),
        })
    }
}

impl Actor for NotifyActor {
    type Context = SyncContext<Self>;
}
