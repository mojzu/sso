use crate::core;
use crate::server::route::auth::reset::PasswordTemplateBody;
use crate::server::{ConfigurationSmtp, Error, SmtpError};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, Transport};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};

// TODO(feature): Improve email templates, formatting.
// TODO(feature): HTML sanitisation for template.
// <https://github.com/rust-ammonia/ammonia>

pub fn send_reset_password(
    smtp: Option<&ConfigurationSmtp>,
    service: &core::Service,
    user: &core::User,
    token: &str,
    template: Option<&PasswordTemplateBody>,
) -> Result<(), Error> {
    let smtp = smtp.ok_or(Error::Smtp(SmtpError::Disabled))?;

    let (subject, text) = match template {
        Some(template) => {
            (template.subject.to_owned(), template.text.to_owned())
        }
        None => (
            format!("{}: Reset Password Request", service.name),
            format!("A reset password request for your email address has been made to {}. If you made this request, follow the link below.", service.name),
        )
    };
    let text = format!(
        "{}\r\n\r\n{}?email={}&reset_password_token={}",
        text, service.url, &user.email, token,
    );
    let email = Email::builder()
        .to((user.email.to_owned(), user.name.to_owned()))
        .from((smtp.user.to_owned(), service.name.to_owned()))
        .subject(subject)
        .text(text)
        .build()
        .map_err(|err| Error::Smtp(SmtpError::Failure(err)))?;

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
