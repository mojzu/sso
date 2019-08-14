//! # Notification Actor
mod email;
mod template;

use crate::core::{Audit, Service, User};
use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use handlebars::Handlebars;

/// ## SMTP Errors
#[derive(Debug, Fail)]
pub enum NotifySmtpError {
    /// Integration disabled.
    #[fail(display = "SmtpError:Disabled")]
    Disabled,
    /// Native TLS error.
    #[fail(display = "SmtpError:NativeTls {}", _0)]
    NativeTls(native_tls::Error),
    /// Lettre email error.
    #[fail(display = "SmtpError:LettreEmail {}", _0)]
    LettreEmail(lettre_email::error::Error),
    /// Lettre error.
    #[fail(display = "SmtpError:Lettre {}", _0)]
    Lettre(lettre::smtp::error::Error),
}

/// ## Notify Errors
#[derive(Debug, Fail)]
pub enum NotifyError {
    /// SMTP error.
    #[fail(display = "NotifyError:Smtp {}", _0)]
    Smtp(NotifySmtpError),
    /// Handlebars template error wrapper.
    #[fail(display = "NotifyError:HandlebarsTemplate {}", _0)]
    HandlebarsTemplate(#[fail(cause)] handlebars::TemplateError),
    /// Handlebars render error wrapper.
    #[fail(display = "NotifyError:HandlebarsRender {}", _0)]
    HandlebarsRender(#[fail(cause)] handlebars::RenderError),
}

/// ## SMTP Options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyExecutorOptionsSmtp {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl NotifyExecutorOptionsSmtp {
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

/// ## Notify Actor Options
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct NotifyExecutorOptions {
    #[builder(default)]
    smtp: Option<NotifyExecutorOptionsSmtp>,
}

/// ## Notify Actor Executor
pub struct NotifyExecutor {
    options: NotifyExecutorOptions,
    registry: Handlebars,
}

impl NotifyExecutor {
    /// Start notifications actor on number of threads with options.
    pub fn start(threads: usize, options: NotifyExecutorOptions) -> Addr<Self> {
        SyncArbiter::start(threads, move || {
            // Register template strings.
            let mut handlebars = Handlebars::new();
            template::register(&mut handlebars).unwrap();

            Self {
                options: options.clone(),
                registry: handlebars,
            }
        })
    }

    /// Returns SMTP provider reference.
    pub fn smtp(&self) -> Result<&NotifyExecutorOptionsSmtp, NotifyError> {
        self.options
            .smtp
            .as_ref()
            .ok_or(NotifyError::Smtp(NotifySmtpError::Disabled))
    }

    /// Returns template registry reference.
    pub fn registry(&self) -> &Handlebars {
        &self.registry
    }
}

impl Actor for NotifyExecutor {
    type Context = SyncContext<Self>;
}

/// ## Reset Password Email Message Data
#[derive(Debug, Deserialize)]
pub struct EmailResetPassword {
    service: Service,
    user: User,
    token: String,
    audit: Option<Audit>,
}

impl EmailResetPassword {
    /// Create new message data.
    pub fn new(service: Service, user: User, token: String, audit: Option<Audit>) -> Self {
        Self {
            service,
            user,
            token,
            audit,
        }
    }
}

impl Message for EmailResetPassword {
    type Result = Result<(), NotifyError>;
}

impl Handler<EmailResetPassword> for NotifyExecutor {
    type Result = Result<(), NotifyError>;

    fn handle(&mut self, msg: EmailResetPassword, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| email::reset_password_handler(smtp, self.registry(), &msg))
            .or_else(warn_on_err)
    }
}

/// ## Update Email Email Message Data
#[derive(Debug, Deserialize)]
pub struct EmailUpdateEmail {
    service: Service,
    user: User,
    old_email: String,
    token: String,
    audit: Option<Audit>,
}

impl EmailUpdateEmail {
    /// Create new message data.
    pub fn new(
        service: Service,
        user: User,
        old_email: String,
        token: String,
        audit: Option<Audit>,
    ) -> Self {
        Self {
            service,
            user,
            old_email,
            token,
            audit,
        }
    }
}

impl Message for EmailUpdateEmail {
    type Result = Result<(), NotifyError>;
}

impl Handler<EmailUpdateEmail> for NotifyExecutor {
    type Result = Result<(), NotifyError>;

    fn handle(&mut self, msg: EmailUpdateEmail, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| email::update_email_handler(smtp, self.registry(), &msg))
            .or_else(warn_on_err)
    }
}

/// ## Update Password Email Message Data
#[derive(Debug, Deserialize)]
pub struct EmailUpdatePassword {
    service: Service,
    user: User,
    token: String,
    audit: Option<Audit>,
}

impl EmailUpdatePassword {
    /// Create new message data.
    pub fn new(service: Service, user: User, token: String, audit: Option<Audit>) -> Self {
        Self {
            service,
            user,
            token,
            audit,
        }
    }
}

impl Message for EmailUpdatePassword {
    type Result = Result<(), NotifyError>;
}

impl Handler<EmailUpdatePassword> for NotifyExecutor {
    type Result = Result<(), NotifyError>;

    fn handle(&mut self, msg: EmailUpdatePassword, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| email::update_password_handler(smtp, self.registry(), &msg))
            .or_else(warn_on_err)
    }
}

fn warn_on_err(err: NotifyError) -> Result<(), NotifyError> {
    warn!("{}", err);
    Ok(())
}
