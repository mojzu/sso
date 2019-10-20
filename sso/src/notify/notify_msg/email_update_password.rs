use crate::{
    AuditMeta, NotifyActor, NotifyActorOptionsSmtp, NotifyError, NotifyResult, Service, User,
};
use actix::{Handler, Message};

/// Update password email message.
#[derive(Debug, Deserialize)]
pub struct EmailUpdatePassword {
    service: Service,
    user: User,
    token: String,
    audit: AuditMeta,
}

impl EmailUpdatePassword {
    /// Create new message data.
    pub fn new(service: Service, user: User, token: String, audit: AuditMeta) -> Self {
        Self {
            service,
            user,
            token,
            audit,
        }
    }
}

impl Message for EmailUpdatePassword {
    type Result = NotifyResult<()>;
}

impl Handler<EmailUpdatePassword> for NotifyActor {
    type Result = NotifyResult<()>;

    fn handle(&mut self, msg: EmailUpdatePassword, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| self.update_password_handler(smtp, &msg))
            .or_else(NotifyActor::warn_on_err)
    }
}

#[derive(Debug, Serialize)]
struct EmailUpdatePasswordQuery {
    email: String,
    token: String,
}

impl NotifyActor {
    fn update_password_handler(
        &self,
        smtp: &NotifyActorOptionsSmtp,
        data: &EmailUpdatePassword,
    ) -> NotifyResult<()> {
        let url = data
            .service
            .provider_local_callback_url(
                "update_password",
                EmailUpdatePasswordQuery {
                    email: data.user.email.to_owned(),
                    token: data.token.to_owned(),
                },
            )
            .map_err(NotifyError::Core)?;

        let parameters = json!({
            "user_email": &data.user.email,
            "url_title": "Revoke Access",
            "url": url.as_str(),
            "service_name": &data.service.name,
            "service_url": &data.service.url,
            "audit": NotifyActor::audit_value(&data.audit),
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
