use crate::{
    NotifyActor, NotifyActorOptionsSmtp, NotifyError, NotifyResult, NotifySmtpError, Service,
};
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient, Transport,
};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};

impl NotifyActor {
    /// Returns SMTP provider reference.
    pub fn smtp(&self) -> NotifyResult<&NotifyActorOptionsSmtp> {
        self.options
            .smtp
            .as_ref()
            .ok_or(NotifyError::Smtp(NotifySmtpError::Disabled))
    }

    /// Send an email using SMTP.
    pub fn smtp_send(
        &self,
        smtp: &NotifyActorOptionsSmtp,
        service: &Service,
        to: String,
        name: String,
        subject: &str,
        text: &str,
    ) -> NotifyResult<()> {
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
}
