mod template;

use crate::core::{Service, User};
use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use handlebars::Handlebars;

/// SMTP errors.
#[derive(Debug, Fail)]
pub enum SmtpError {
    /// Integration disabled.
    #[fail(display = "SmtpError::Disabled")]
    Disabled,
    /// Native TLS error.
    #[fail(display = "SmtpError::NativeTls {}", _0)]
    NativeTls(native_tls::Error),
    /// Lettre email error.
    #[fail(display = "SmtpError::LettreEmail {}", _0)]
    LettreEmail(lettre_email::error::Error),
    /// Lettre error.
    #[fail(display = "SmtpError::Lettre {}", _0)]
    Lettre(lettre::smtp::error::Error),
}

/// Notify errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// SMTP error.
    #[fail(display = "NotifyError::Smtp {}", _0)]
    Smtp(SmtpError),
    /// Handlebars template render error wrapper.
    #[fail(display = "NotifyError::HandlebarsTemplateRender {}", _0)]
    HandlebarsTemplateRender(#[fail(cause)] handlebars::TemplateRenderError),
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSmtp {
    host: String,
    port: u16,
    user: String,
    password: String,
}

/// Notify configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    smtp: Option<ConfigurationSmtp>,
}

impl Configuration {
    /// Create new configuration.
    pub fn new() -> Self {
        Configuration { smtp: None }
    }

    // Set SMTP provider.
    pub fn set_smtp(
        &mut self,
        host: String,
        port: u16,
        user: String,
        password: String,
    ) -> &mut Self {
        self.smtp = Some(ConfigurationSmtp {
            host,
            port,
            user,
            password,
        });
        self
    }
}

/// Notify actor.
pub struct NotifyExecutor {
    configuration: Configuration,
    handlebars: Handlebars,
}

impl NotifyExecutor {
    /// Start notify actor.
    pub fn start(threads: usize, configuration: &Configuration) -> Addr<NotifyExecutor> {
        let configuration = configuration.clone();
        SyncArbiter::start(threads, move || {
            // Register template strings.
            let mut handlebars = Handlebars::new();
            template::register(&mut handlebars);

            NotifyExecutor {
                configuration: configuration.clone(),
                handlebars,
            }
        })
    }

    /// Configured SMTP provider reference.
    pub fn smtp(&self) -> Option<&ConfigurationSmtp> {
        self.configuration.smtp.as_ref()
    }
}

impl Actor for NotifyExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Deserialize)]
pub struct EmailResetPassword {
    pub service: Service,
    pub user: User,
    pub token: String,
}

impl Message for EmailResetPassword {
    type Result = Result<(), Error>;
}

impl Handler<EmailResetPassword> for NotifyExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: EmailResetPassword, _: &mut Self::Context) -> Self::Result {
        // TODO(refactor): Implement this.
        // smtp::send_reset_password(data.configuration().smtp(), &service, &user, &token).or_else(
        //     |err| {
        //         warn!("{}", err);
        //         Ok(())
        //     },
        // )
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailUpdateEmail {
    pub service: Service,
    pub user: User,
    pub old_email: String,
    pub token: String,
}

impl Message for EmailUpdateEmail {
    type Result = Result<(), Error>;
}

impl Handler<EmailUpdateEmail> for NotifyExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: EmailUpdateEmail, _: &mut Self::Context) -> Self::Result {
        // TODO(refactor): Implement this.
        // smtp::send_update_email(
        //     data.configuration().smtp(),
        //     &service,
        //     &user,
        //     &old_email,
        //     &token,
        // )
        // .or_else(|err| {
        //     warn!("{}", err);
        //     Ok(())
        // })
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct EmailUpdatePassword {
    pub service: Service,
    pub user: User,
    pub token: String,
}

impl Message for EmailUpdatePassword {
    type Result = Result<(), Error>;
}

impl Handler<EmailUpdatePassword> for NotifyExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: EmailUpdatePassword, _: &mut Self::Context) -> Self::Result {
        // TODO(refactor): Implement this.
        // smtp::send_update_password(data.configuration().smtp(), &service, &user, &token)
        //     .or_else(|err| {
        //         warn!("{}", err);
        //         Ok(())
        //     })
        Ok(())
    }
}
