use crate::{DriverError, DriverResult, Env};
use http::{header, HeaderMap};
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient,
};
use native_tls::{Protocol, TlsConnector};
use reqwest::Client;
use std::fs::create_dir_all;

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
    pwned_passwords_enabled: bool,
    /// Enabled Traefik forward authentication.
    traefik_enabled: bool,
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
    /// Returns new server options.
    pub fn new<UA>(user_agent: UA, pwned_passwords_enabled: bool, traefik_enabled: bool) -> Self
    where
        UA: Into<String>,
    {
        Self {
            user_agent: user_agent.into(),
            pwned_passwords_enabled,
            traefik_enabled,
            access_token_expires: 3_600,
            refresh_token_expires: 86_400,
            revoke_token_expires: 604_800,
            smtp_transport: None,
            smtp_file_transport: None,
            github: None,
            microsoft: None,
        }
    }

    pub fn from_env<T: AsRef<str>>(
        user_agent_name: T,
        pwned_passwords_enabled_name: T,
        traefik_enabled_name: T,
    ) -> Self {
        let user_agent = Env::string_opt(user_agent_name.as_ref()).unwrap_or("sso".to_owned());
        let pwned_passwords_enabled = Env::value_opt::<bool>(pwned_passwords_enabled_name.as_ref())
            .expect("Failed to read Pwned Passwords enabled environment variable.")
            .unwrap_or(false);
        let traefik_enabled = Env::value_opt::<bool>(traefik_enabled_name.as_ref())
            .expect("Failed to read Traefik enabled environment variable.")
            .unwrap_or(false);

        Self::new(user_agent, pwned_passwords_enabled, traefik_enabled)
    }

    /// Set SMTP transport options.
    pub fn smtp_transport(mut self, smtp_transport: Option<ServerOptionsSmtp>) -> Self {
        self.smtp_transport = smtp_transport;
        self
    }

    /// Read SMTP environment variables into options.
    ///
    /// If no variables are defined, returns None. Else all variables
    /// are required and an error message logged for each missing variable.
    pub fn smtp_transport_from_env<T: AsRef<str>>(
        self,
        host_name: T,
        port_name: T,
        user_name: T,
        password_name: T,
    ) -> Self {
        let transport = if Env::has_any_name(&[
            host_name.as_ref(),
            port_name.as_ref(),
            user_name.as_ref(),
            password_name.as_ref(),
        ]) {
            let host = Env::string(host_name.as_ref())
                .expect("Failed to read SMTP host environment variable.");
            let port = Env::value::<u16>(port_name.as_ref())
                .expect("Failed to read SMTP port environment variable.");
            let user = Env::string(user_name.as_ref())
                .expect("Failed to read SMTP user environment variable.");
            let password = Env::string(password_name.as_ref())
                .expect("Failed to read SMTP password environment variable.");

            Some(ServerOptionsSmtp::new(host, port, user, password))
        } else {
            None
        };
        self.smtp_transport(transport)
    }

    /// Set SMTP file transport.
    pub fn smtp_file_transport(mut self, smtp_file_transport: Option<String>) -> Self {
        self.smtp_file_transport = smtp_file_transport;
        self
    }

    // Create directory for SMTP file transport.
    pub fn smtp_file_transport_from_env<T: AsRef<str>>(self, file_name: T) -> Self {
        let transport =
            Env::string(file_name.as_ref()).expect("Failed to read SMTP file environment variable");
        create_dir_all(&transport).expect("Failed to create SMTP file transport directory");
        self.smtp_file_transport(Some(transport))
    }

    /// Set Github provider.
    pub fn github(mut self, github: Option<ServerOptionsProvider>) -> Self {
        self.github = github;
        self
    }

    /// Read OAuth2 environment variables into options.
    ///
    /// If no variables are defined, returns None. Else all variables
    /// are required and an error message logged for each missing variable.
    pub fn github_from_env<T: AsRef<str>>(self, client_id_name: T, client_secret_name: T) -> Self {
        let provider = if Env::has_any_name(&[client_id_name.as_ref(), client_secret_name.as_ref()])
        {
            let client_id = Env::string(client_id_name.as_ref())
                .expect("Failed to read Github client ID environment variable");
            let client_secret = Env::string(client_secret_name.as_ref())
                .expect("Failed to read Github client secret environment variable");

            Some(ServerOptionsProvider::new(client_id, client_secret))
        } else {
            None
        };
        self.github(provider)
    }

    /// Set Microsoft provider.
    pub fn microsoft(mut self, microsoft: Option<ServerOptionsProvider>) -> Self {
        self.microsoft = microsoft;
        self
    }

    /// Read OAuth2 environment variables into options.
    ///
    /// If no variables are defined, returns None. Else all variables
    /// are required and an error message logged for each missing variable.
    pub fn microsoft_from_env<T: AsRef<str>>(
        self,
        client_id_name: T,
        client_secret_name: T,
    ) -> Self {
        let provider = if Env::has_any_name(&[client_id_name.as_ref(), client_secret_name.as_ref()])
        {
            let client_id = Env::string(client_id_name.as_ref())
                .expect("Failed to read Microsoft client ID environment variable");
            let client_secret = Env::string(client_secret_name.as_ref())
                .expect("Failed to read Microsoft client secret environment variable");

            Some(ServerOptionsProvider::new(client_id, client_secret))
        } else {
            None
        };
        self.microsoft(provider)
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

    /// Returns Pwned Passwords integration enabled flag.
    pub fn pwned_passwords_enabled(&self) -> bool {
        self.pwned_passwords_enabled
    }

    /// Returns Traefik integration enabled flag.
    pub fn traefik_enabled(&self) -> bool {
        self.traefik_enabled
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
