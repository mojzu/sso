use crate::core;
use crate::server::template;
use crate::server::{ConfigurationSmtp, Error, SmtpError};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, Transport};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};

// TODO(feature): Improve email templates, formatting, style (red button for revoke).
// Locale parameter or user column for translations?

pub fn send_reset_password(
    smtp: Option<&ConfigurationSmtp>,
    service: &core::Service,
    user: &core::User,
    token: &str,
) -> Result<(), Error> {
    let subject = "Password Reset Request";
    let text = format!("A request has been made to reset the password for your email address ({}). If you made this request, click the link below.", &user.email);
    let callback_data = &[("email", user.email.as_ref()), ("token", token)];
    let url = service.callback_url("reset_password", callback_data);

    let parameters = template::Email::new(
        &subject,
        &text,
        "Reset Password",
        url.as_str(),
        &service.name,
        &service.url,
    );
    let (text, html) = template::email(&parameters)?;
    send(
        smtp,
        service,
        user.email.to_owned(),
        user.name.to_owned(),
        subject,
        &text,
        html,
    )
}

pub fn send_update_email(
    smtp: Option<&ConfigurationSmtp>,
    service: &core::Service,
    user: &core::User,
    old_email: &str,
    token: &str,
) -> Result<(), Error> {
    let subject = "Update Email Request";
    let text = format!("A request has been made to update the email address for your user to {} (from {}). If you did not make this request, click the link below to revoke access.", &user.email, old_email);
    let callback_data = &[
        ("email", user.email.as_ref()),
        ("old_email", old_email),
        ("token", token),
    ];
    let url = service.callback_url("update_email", callback_data);

    let parameters = template::Email::new(
        subject,
        &text,
        "Revoke Access",
        url.as_str(),
        &service.name,
        &service.url,
    );
    let (text, html) = template::email(&parameters)?;
    send(
        smtp,
        service,
        old_email.to_owned(),
        user.name.to_owned(),
        subject,
        &text,
        html,
    )
}

pub fn send_update_password(
    smtp: Option<&ConfigurationSmtp>,
    service: &core::Service,
    user: &core::User,
    token: &str,
) -> Result<(), Error> {
    let subject = "Update Password Request";
    let text = format!("A request has been made to update the password for your email address ({}). If you made this request, click the link below.", &user.email);
    let callback_data = &[("email", user.email.as_ref()), ("token", token)];
    let url = service.callback_url("update_password", callback_data);

    let parameters = template::Email::new(
        subject,
        &text,
        "Revoke Access",
        url.as_str(),
        &service.name,
        &service.url,
    );
    let (text, html) = template::email(&parameters)?;
    send(
        smtp,
        service,
        user.email.to_owned(),
        user.name.to_owned(),
        subject,
        &text,
        html,
    )
}

fn send(
    smtp: Option<&ConfigurationSmtp>,
    service: &core::Service,
    to: String,
    name: String,
    subject: &str,
    text: &str,
    html: String,
) -> Result<(), Error> {
    let smtp = smtp.ok_or(Error::Smtp(SmtpError::Disabled))?;
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
    .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
    .transport();

    let result = mailer
        .send(email.into())
        .map_err(|err| Error::Smtp(SmtpError::Lettre(err)))
        .map(|_res| ());
    mailer.close();
    result
}
