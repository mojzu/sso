use crate::{DriverError, DriverResult};
use http::{header, HeaderMap};
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient,
};
use native_tls::{Protocol, TlsConnector};
use reqwest::Client;

/// gRPC server authentication provider options.
#[derive(Debug, Clone)]
pub struct ServerOptionsProvider {
    pub client_id: String,
    pub client_secret: String,
}

impl ServerOptionsProvider {
    /// Returns new `ServerOptionsProvider`.
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }
}

/// gRPC server SMTP transport options.
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
    /// Access token expiry time in seconds.
    access_token_expires: i64,
    /// Refresh token expiry time in seconds.
    refresh_token_expires: i64,
    /// Revoke token expiry time in seconds.
    revoke_token_expires: i64,
    /// SMTP transport.
    smtp_transport: Option<ServerOptionsSmtp>,
    /// SMTP file transport.
    ///
    /// Writes emails to files in directory, if server settings
    /// are defined this is ignored.
    smtp_file_transport: Option<String>,
    /// Github provider.
    github: Option<ServerOptionsProvider>,
    /// Microsoft provider.
    microsoft: Option<ServerOptionsProvider>,
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
            access_token_expires: 3_600,
            refresh_token_expires: 86_400,
            revoke_token_expires: 604_800,
            smtp_transport: None,
            smtp_file_transport: None,
            github: None,
            microsoft: None,
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

    /// Set Github provider.
    pub fn github(mut self, github: Option<ServerOptionsProvider>) -> Self {
        self.github = github;
        self
    }

    /// Set Microsoft provider.
    pub fn microsoft(mut self, microsoft: Option<ServerOptionsProvider>) -> Self {
        self.microsoft = microsoft;
        self
    }

    /// Returns asynchronous reqwest `Client` built from options.
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

    /// Returns access token expiry value.
    pub fn access_token_expires(&self) -> i64 {
        self.access_token_expires
    }

    /// Returns refresh token expiry value.
    pub fn refresh_token_expires(&self) -> i64 {
        self.refresh_token_expires
    }

    /// Returns revoke token expiry value.
    pub fn revoke_token_expires(&self) -> i64 {
        self.revoke_token_expires
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

    /// Returns provider GitHub OAuth2 common arguments.
    pub(crate) fn github_oauth2_args(&self) -> ServerProviderOauth2Args {
        ServerProviderOauth2Args::new(
            self.github.clone(),
            self.access_token_expires(),
            self.refresh_token_expires(),
        )
    }

    /// Returns provider Microsoft OAuth2 common arguments.
    pub(crate) fn microsoft_oauth2_args(&self) -> ServerProviderOauth2Args {
        ServerProviderOauth2Args::new(
            self.microsoft.clone(),
            self.access_token_expires(),
            self.refresh_token_expires(),
        )
    }
}

/// Authentication provider OAuth2 common arguments.
#[derive(Debug)]
pub(crate) struct ServerProviderOauth2Args {
    pub provider: Option<ServerOptionsProvider>,
    pub access_token_expires: i64,
    pub refresh_token_expires: i64,
}

impl ServerProviderOauth2Args {
    pub fn new(
        provider: Option<ServerOptionsProvider>,
        access_token_expires: i64,
        refresh_token_expires: i64,
    ) -> Self {
        Self {
            provider,
            access_token_expires,
            refresh_token_expires,
        }
    }
}
