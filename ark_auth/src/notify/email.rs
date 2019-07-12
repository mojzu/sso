use crate::core::Service;
use crate::notify::template;
use crate::notify::{
    ConfigurationSmtp, EmailResetPassword, EmailUpdateEmail, EmailUpdatePassword, Error, SmtpError,
};
use handlebars::Handlebars;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, Transport};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};

// TODO(refactor): Improve email templates, formatting, style (red button for revoke).
// Locale parameter or user column for translations?

pub fn reset_password_handler(
    smtp: &ConfigurationSmtp,
    registry: &Handlebars,
    data: &EmailResetPassword,
) -> Result<(), Error> {
    let subject = "Password Reset Request";
    let text = format!("A request has been made to reset the password for your email address ({}). If you made this request, click the link below.", &data.user.email);
    let url_text = "Reset Password".to_owned();
    let callback_data = &[("email", &data.user.email), ("token", &data.token)];
    let url = data.service.callback_url("reset_password", callback_data);

    let parameters = template::Email {
        title: subject.to_owned(),
        text,
        url_text,
        url: url.as_str().to_owned(),
        service_name: data.service.name.clone(),
        service_url: data.service.url.clone(),
    };
    let (text, html) = template::email(registry, &parameters)?;
    send(
        smtp,
        &data.service,
        data.user.email.to_owned(),
        data.user.name.to_owned(),
        subject,
        &text,
        html,
    )
}

pub fn update_email_handler(
    smtp: &ConfigurationSmtp,
    registry: &Handlebars,
    data: &EmailUpdateEmail,
) -> Result<(), Error> {
    let subject = "Update Email Request";
    let text = format!("A request has been made to update the email address for your user to {} (from {}). If you did not make this request, click the link below to revoke access.", &data.user.email, &data.old_email);
    let url_text = "Revoke Access".to_owned();
    let callback_data = &[
        ("email", &data.user.email),
        ("old_email", &data.old_email),
        ("token", &data.token),
    ];
    let url = data.service.callback_url("update_email", callback_data);

    let parameters = template::Email {
        title: subject.to_owned(),
        text,
        url_text,
        url: url.as_str().to_owned(),
        service_name: data.service.name.clone(),
        service_url: data.service.url.clone(),
    };
    let (text, html) = template::email(registry, &parameters)?;
    send(
        smtp,
        &data.service,
        data.old_email.to_owned(),
        data.user.name.to_owned(),
        subject,
        &text,
        html,
    )
}

pub fn update_password_handler(
    smtp: &ConfigurationSmtp,
    registry: &Handlebars,
    data: &EmailUpdatePassword,
) -> Result<(), Error> {
    let subject = "Update Password Request";
    let text = format!("A request has been made to update the password for your email address ({}). If you made this request, click the link below.", &data.user.email);
    let url_text = "Revoke Access".to_owned();
    let callback_data = &[("email", &data.user.email), ("token", &data.token)];
    let url = data.service.callback_url("update_password", callback_data);

    let parameters = template::Email {
        title: subject.to_owned(),
        text,
        url_text,
        url: url.as_str().to_owned(),
        service_name: data.service.name.clone(),
        service_url: data.service.url.clone(),
    };
    let (text, html) = template::email(registry, &parameters)?;
    send(
        smtp,
        &data.service,
        data.user.email.to_owned(),
        data.user.name.to_owned(),
        subject,
        &text,
        html,
    )
}

fn send(
    smtp: &ConfigurationSmtp,
    service: &Service,
    to: String,
    name: String,
    subject: &str,
    text: &str,
    html: String,
) -> Result<(), Error> {
    let email = Email::builder()
        .to((to, name))
        .from((smtp.user.to_owned(), service.name.to_owned()))
        .subject(subject)
        .html(html)
        .text(text)
        .build()
        .map_err(|err| Error::Smtp(SmtpError::LettreEmail(err)))?;

    let mut tls_builder = TlsConnector::builder();
    tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
    let tls_parameters = ClientTlsParameters::new(
        smtp.host.to_owned(),
        tls_builder
            .build()
            .map_err(|err| Error::Smtp(SmtpError::NativeTls(err)))?,
    );
    let mut mailer = SmtpClient::new(
        (smtp.host.as_ref(), smtp.port),
        ClientSecurity::Required(tls_parameters),
    )
    .map_err(|err| Error::Smtp(SmtpError::Lettre(err)))?
    .authentication_mechanism(Mechanism::Login)
    .credentials(Credentials::new(
        smtp.user.to_owned(),
        smtp.password.to_owned(),
    ))
    .transport();

    let result = mailer
        .send(email.into())
        .map_err(|err| Error::Smtp(SmtpError::Lettre(err)))
        .map(|_res| ());
    mailer.close();
    result
}
