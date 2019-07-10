use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};

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

pub struct NotifyExecutor {}

impl NotifyExecutor {
    pub fn start(threads: usize) -> Addr<NotifyExecutor> {
        SyncArbiter::start(threads, move || NotifyExecutor {})
    }
}

impl Actor for NotifyExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Debug, Deserialize)]
pub struct EmailAction {
    pub email: String,
    pub password: String,
}

impl Message for EmailAction {
    type Result = Result<(), Error>;
}

impl Handler<EmailAction> for NotifyExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: EmailAction, _: &mut Self::Context) -> Self::Result {
        // TODO(refactor): Implement this.
        Ok(())
    }
}
