use crate::{
    AuditMeta, NotifyActor, NotifyActorOptionsSmtp, NotifyError, NotifyResult, Service, User,
};
use actix::{Handler, Message};

/// Reset password email message.
#[derive(Debug, Deserialize)]
pub struct EmailResetPassword {
    service: Service,
    user: User,
    token: String,
    audit: AuditMeta,
}

impl EmailResetPassword {
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

impl Message for EmailResetPassword {
    type Result = NotifyResult<()>;
}

impl Handler<EmailResetPassword> for NotifyActor {
    type Result = NotifyResult<()>;

    fn handle(&mut self, msg: EmailResetPassword, _: &mut Self::Context) -> Self::Result {
        self.smtp()
            .and_then(|smtp| self.reset_password_handler(smtp, &msg))
            .or_else(NotifyActor::warn_on_err)
    }
}

#[derive(Debug, Serialize)]
struct EmailResetPasswordQuery {
    email: String,
    token: String,
}

impl NotifyActor {
    fn reset_password_handler(
        &self,
        smtp: &NotifyActorOptionsSmtp,
        data: &EmailResetPassword,
    ) -> NotifyResult<()> {
        let url = data
            .service
            .provider_local_callback_url(
                "reset_password",
                EmailResetPasswordQuery {
                    email: data.user.email.to_owned(),
                    token: data.token.to_owned(),
                },
            )
            .map_err(NotifyError::Driver)?;

        let parameters = json!({
            "user_email": &data.user.email,
            "url_title": "Reset Password",
            "url": url.as_str(),
            "service_name": &data.service.name,
            "service_url": &data.service.url,
            "audit": NotifyActor::audit_value(&data.audit),
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
