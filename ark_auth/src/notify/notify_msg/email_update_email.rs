use crate::{Audit, NotifyActor, NotifyActorOptionsSmtp, NotifyError, NotifyResult, Service, User};
use actix::{Handler, Message};

/// Update email email message.
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
    type Result = NotifyResult<()>;
}

impl Handler<EmailUpdateEmail> for NotifyActor {
    type Result = NotifyResult<()>;

    fn handle(&mut self, msg: EmailUpdateEmail, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| self.update_email_handler(smtp, &msg))
            .or_else(NotifyActor::warn_on_err)
    }
}

impl NotifyActor {
    fn update_email_handler(
        &self,
        smtp: &NotifyActorOptionsSmtp,
        data: &EmailUpdateEmail,
    ) -> NotifyResult<()> {
        let callback_data = &[
            ("email", &data.user.email),
            ("old_email", &data.old_email),
            ("token", &data.token),
        ];
        let url = data
            .service
            .callback_url("update_email", callback_data)
            .map_err(NotifyError::Core)?;

        let parameters = json!({
            "user_old_email": &data.old_email,
            "user_email": &data.user.email,
            "url_title": "Revoke Access",
            "url": url.as_str(),
            "service_name": &data.service.name,
            "service_url": &data.service.url,
            "audit": NotifyActor::audit_value(data.audit.as_ref()),
        });
        let text = self.template_email_update_email(&parameters)?;

        self.smtp_send(
            smtp,
            &data.service,
            data.old_email.to_owned(),
            data.user.name.to_owned(),
            "Email Address Updated",
            &text,
        )
    }
}
