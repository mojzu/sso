use crate::{DriverError, DriverResult};
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient,
};
use native_tls::{Protocol, TlsConnector};

/// Server SMTP options.
#[derive(Debug, Clone)]
pub struct ServerOptionsSmtp {
    host: String,
    port: u16,
    user: String,
    password: String,
}

impl ServerOptionsSmtp {
    /// Create new SMTP options.
    pub fn new(host: String, port: u16, user: String, password: String) -> Self {
        Self {
            host,
            port,
            user,
            password,
        }
    }
}

/// gRPC server options.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// SMTP transport.
    smtp_transport: Option<ServerOptionsSmtp>,
    /// SMTP file transport.
    smtp_file_transport: Option<String>,
}

impl ServerOptions {
    /// Returns new `ServerOptions`.
    pub fn new() -> Self {
        Self {
            smtp_transport: None,
            smtp_file_transport: None,
        }
    }

    /// Set SMTP transport options.
    pub fn smtp_transport(mut self, smtp_transport: Option<ServerOptionsSmtp>) -> Self {
        self.smtp_transport = smtp_transport;
        self
    }

    /// Set SMTP file transport.
    pub fn smtp_file_transport(mut self, smtp_file_transport: Option<String>) -> Self {
        self.smtp_file_transport = smtp_file_transport;
        self
    }

    /// Returns `SmtpClient` built from options.
    pub fn smtp_client(&self) -> DriverResult<Option<SmtpClient>> {
        if let Some(smtp) = self.smtp_transport.as_ref() {
            let mut tls_builder = TlsConnector::builder();
            tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
            let tls_parameters = ClientTlsParameters::new(
                smtp.host.to_owned(),
                tls_builder.build().map_err(DriverError::NativeTls)?,
            );

            let client = SmtpClient::new(
                (smtp.host.as_ref(), smtp.port),
                ClientSecurity::Required(tls_parameters),
            )
            .map_err(DriverError::Lettre)?
            .authentication_mechanism(Mechanism::Login)
            .credentials(Credentials::new(
                smtp.user.to_owned(),
                smtp.password.to_owned(),
            ));
            Ok(Some(client))
        } else {
            Ok(None)
        }
    }

    /// Returns default `From` email address from SMTP options.
    pub fn smtp_from_email(&self) -> Option<String> {
        self.smtp_transport.as_ref().map(|x| x.user.to_owned())
    }

    /// Returns SMTP file transport directory.
    pub fn smtp_file(&self) -> Option<String> {
        self.smtp_file_transport.as_ref().map(|x| x.to_owned())
    }
}
