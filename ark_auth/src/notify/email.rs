use crate::core::{Audit, Service};
use crate::notify::template;
use crate::notify::{
    EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword, NotifyError,
    NotifyExecutorOptionsSmtp, NotifySmtpError,
};
use handlebars::Handlebars;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, Transport};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
use serde_json::Value;

pub fn reset_password_handler(
    smtp: &NotifyExecutorOptionsSmtp,
    registry: &Handlebars,
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
        "audit": audit_value(data.audit.as_ref()),
    });
    let text = template::email_reset_password(registry, &parameters)?;

    send(
        smtp,
        &data.service,
        data.user.email.to_owned(),
        data.user.name.to_owned(),
        "Password Reset Request",
        &text,
    )
}

pub fn update_email_handler(
    smtp: &NotifyExecutorOptionsSmtp,
    registry: &Handlebars,
    data: &EmailUpdateEmail,
) -> Result<(), NotifyError> {
    let callback_data = &[
        ("email", &data.user.email),
        ("old_email", &data.old_email),
        ("token", &data.token),
    ];
    let url = data.service.callback_url("update_email", callback_data);

    let parameters = json!({
        "user_old_email": &data.old_email,
        "user_email": &data.user.email,
        "url_title": "Revoke Access",
        "url": url.as_str(),
        "service_name": &data.service.name,
        "service_url": &data.service.url,
        "audit": audit_value(data.audit.as_ref()),
    });
    let text = template::email_update_email(registry, &parameters)?;

    send(
        smtp,
        &data.service,
        data.old_email.to_owned(),
        data.user.name.to_owned(),
        "Email Address Updated",
        &text,
    )
}

pub fn update_password_handler(
    smtp: &NotifyExecutorOptionsSmtp,
    registry: &Handlebars,
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
        "audit": audit_value(data.audit.as_ref()),
    });
    let text = template::email_update_password(registry, &parameters)?;

    send(
        smtp,
        &data.service,
        data.user.email.to_owned(),
        data.user.name.to_owned(),
        "Password Updated",
        &text,
    )
}

fn audit_value(audit: Option<&Audit>) -> Option<Value> {
    match audit {
        Some(audit) => Some(json!({
            "id": audit.id,
            "created_at": audit.created_at,
            "user_agent": audit.user_agent,
            "remote": audit.remote,
            "forwarded": audit.forwarded,
        })),
        None => None,
    }
}

fn send(
    smtp: &NotifyExecutorOptionsSmtp,
    service: &Service,
    to: String,
    name: String,
    subject: &str,
    text: &str,
) -> Result<(), NotifyError> {
    let email = Email::builder()
        .to((to, name))
        .from((smtp.user.to_owned(), service.name.to_owned()))
        .subject(subject)
        .text(text)
        .build()
        .map_err(|err| NotifyError::Smtp(NotifySmtpError::LettreEmail(err)))?;

    let mut tls_builder = TlsConnector::builder();
    tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
    let tls_parameters = ClientTlsParameters::new(
        smtp.host.to_owned(),
        tls_builder
            .build()
            .map_err(|err| NotifyError::Smtp(NotifySmtpError::NativeTls(err)))?,
    );
    let mut mailer = SmtpClient::new(
        (smtp.host.as_ref(), smtp.port),
        ClientSecurity::Required(tls_parameters),
    )
    .map_err(|err| NotifyError::Smtp(NotifySmtpError::Lettre(err)))?
    .authentication_mechanism(Mechanism::Login)
    .credentials(Credentials::new(
        smtp.user.to_owned(),
        smtp.password.to_owned(),
    ))
    .transport();

    let result = mailer
        .send(email.into())
        .map_err(|err| NotifyError::Smtp(NotifySmtpError::Lettre(err)))
        .map(|_res| ());
    mailer.close();
    result
}
