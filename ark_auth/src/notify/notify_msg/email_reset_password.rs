use crate::{Audit, NotifyActor, NotifyActorOptionsSmtp, NotifyError, Service, User};
use actix::{Handler, Message};

/// Reset password email message.
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

impl Handler<EmailResetPassword> for NotifyActor {
    type Result = Result<(), NotifyError>;

    fn handle(&mut self, msg: EmailResetPassword, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| self.reset_password_handler(smtp, &msg))
            .or_else(NotifyActor::warn_on_err)
    }
}

impl NotifyActor {
    fn reset_password_handler(
        &self,
        smtp: &NotifyActorOptionsSmtp,
        data: &EmailResetPassword,
    ) -> Result<(), NotifyError> {
        let callback_data = &[("email", &data.user.email), ("token", &data.token)];
        let url = data.service.callback_url("reset_password", callback_data);

        let parameters = json!({
            "user_email": &data.user.email,
            "url_title": "Reset Password",
            "url": url.as_str(),
            "service_name": &data.service.name,
            "service_url": &data.service.url,
            "audit": NotifyActor::audit_value(data.audit.as_ref()),
        });
        let text = self.template_email_reset_password(&parameters)?;

        self.smtp_send(
            smtp,
            &data.service,
            data.user.email.to_owned(),
            data.user.name.to_owned(),
            "Password Reset Request",
            &text,
        )
    }
}
