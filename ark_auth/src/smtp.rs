use crate::{core, server, server::ConfigurationSmtp};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::ConnectionReuseParameters;
use lettre::{ClientSecurity, ClientTlsParameters, SmtpClient, Transport};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
use crate::server::auth::reset::PasswordTemplateBody;

pub fn send_reset_password(
    smtp: Option<&ConfigurationSmtp>,
    service: &core::Service,
    user: &core::User,
    token: &str,
    template: Option<&PasswordTemplateBody>,
) -> Result<(), server::Error> {
    // TODO(refactor): Implement this.
    unimplemented!();
}

// pub fn send_reset_password(
//     smtp: Option<&ApiConfigSmtp>,
//     user: &AuthUser,
//     service: &AuthService,
//     token: &str,
// ) -> Result<(), ApiError> {
//     let smtp = smtp.ok_or(ApiError::Unwrap("smtp settings undefined"))?;

//     let email_subject = format!("{}: Reset Password Request", service.service_name);
//     let email_url = format!(
//         "{}?email={}&reset_password_token={}",
//         service.service_url, &user.user_email, token
//     );
//     let email = Email::builder()
//         .to((user.user_email.to_owned(), user.user_name.to_owned()))
//         // TODO(refactor): SMTP name configuration.
//         .from(smtp.user.to_owned())
//         .subject(email_subject)
//         // TODO(feature): HTML templates.
//         .text(email_url)
//         .build()
//         .map_err(|_e| ApiError::Unwrap("failed to build email"))?;

//     let mut tls_builder = TlsConnector::builder();
//     tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
//     let tls_parameters = ClientTlsParameters::new(
//         smtp.host.to_owned(),
//         tls_builder
//             .build()
//             .map_err(|_e| ApiError::Unwrap("failed to build tls settings"))?,
//     );
//     let mut mailer = SmtpClient::new(
//         (smtp.host.as_ref(), smtp.port),
//         ClientSecurity::Required(tls_parameters),
//     )
//     .map_err(|_e| ApiError::Unwrap("failed to create client"))?
//     .authentication_mechanism(Mechanism::Login)
//     .credentials(Credentials::new(
//         smtp.user.to_owned(),
//         smtp.password.to_owned(),
//     ))
//     .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
//     .transport();

//     let result = mailer
//         .send(email.into())
//         .map_err(|e| {
//             error!("e = {}", e);
//             ApiError::Unwrap("failed to send email")
//         })
//         .map(|_r| ());
//     mailer.close();
//     result
// }
