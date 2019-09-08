use crate::{Audit, NotifyActor, NotifyActorOptionsSmtp, NotifyError, Service, User};
use actix::{Handler, Message};

/// Update password email message.
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

impl Handler<EmailUpdatePassword> for NotifyActor {
    type Result = Result<(), NotifyError>;

    fn handle(&mut self, msg: EmailUpdatePassword, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| self.update_password_handler(smtp, &msg))
            .or_else(NotifyActor::warn_on_err)
    }
}

impl NotifyActor {
    fn update_password_handler(
        &self,
        smtp: &NotifyActorOptionsSmtp,
        data: &EmailUpdatePassword,
    ) -> Result<(), NotifyError> {
        let callback_data = &[("email", &data.user.email), ("token", &data.token)];
        let url = data.service.callback_url("update_password", callback_data);

        let parameters = json!({
            "user_email": &data.user.email,
            "url_title": "Revoke Access",
            "url": url.as_str(),
            "service_name": &data.service.name,
            "service_url": &data.service.url,
            "audit": NotifyActor::audit_value(data.audit.as_ref()),
        });
        let text = self.template_email_update_password(&parameters)?;

        self.smtp_send(
            smtp,
            &data.service,
            data.user.email.to_owned(),
            data.user.name.to_owned(),
            "Password Updated",
            &text,
        )
    }
}
