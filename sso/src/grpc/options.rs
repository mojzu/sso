use crate::{DriverError, DriverResult};
use http01::{header, HeaderMap};
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient,
};
use native_tls::{Protocol, TlsConnector};
use reqwest::Client;

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
    /// User agent for outgoing HTTP requests.
    user_agent: String,
    /// Enable Pwned Passwords API to check passwords.
    /// API keys may be required in the future to use this API.
    password_pwned_enabled: bool,
    /// SMTP transport.
    smtp_transport: Option<ServerOptionsSmtp>,
    /// SMTP file transport.
    smtp_file_transport: Option<String>,
}

impl ServerOptions {
    /// Returns new `ServerOptions`.
    pub fn new<UA>(user_agent: UA, password_pwned_enabled: bool) -> Self
    where
        UA: Into<String>,
    {
        Self {
            user_agent: user_agent.into(),
            password_pwned_enabled,
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

    /// Returns synchronous reqwest `Client` built from options.
    pub fn client(&self) -> DriverResult<Client> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, self.user_agent.parse().unwrap());
        Client::builder()
            .use_rustls_tls()
            .default_headers(headers)
            .build()
            .map_err(DriverError::Reqwest)
    }

    /// Returns password pwned enabled flag.
    pub fn password_pwned_enabled(&self) -> bool {
        self.password_pwned_enabled
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
