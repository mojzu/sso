pub mod actix_web_middleware;
mod route;

use crate::{
    api::{AuthProviderOauth2, AuthProviderOauth2Args},
    Driver, DriverError, DriverResult, Metrics, TemplateEmail,
};
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use http::{header, HeaderMap};
use lettre::{
    file::FileTransport,
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient, Transport,
};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
use reqwest::{r#async::Client as AsyncClient, Client as SyncClient};
use rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    AllowAnyAuthenticatedClient, NoClientAuth, RootCertStore, ServerConfig,
};
use serde::Serialize;
use std::{fs::File, io::BufReader, path::PathBuf};

/// Server options provider options.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServerOptionsProvider {
    oauth2: Option<AuthProviderOauth2>,
}

impl ServerOptionsProvider {
    /// New server options provider.
    pub fn new(oauth2: Option<AuthProviderOauth2>) -> Self {
        Self { oauth2 }
    }
}

/// Server options provider group options.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServerOptionsProviderGroup {
    github: ServerOptionsProvider,
    microsoft: ServerOptionsProvider,
}

impl ServerOptionsProviderGroup {
    /// New server provider group options.
    pub fn new(github: ServerOptionsProvider, microsoft: ServerOptionsProvider) -> Self {
        Self { github, microsoft }
    }
}

/// Server Rustls options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOptionsRustls {
    crt_pem: String,
    key_pem: String,
    client_pem: Option<String>,
}

impl ServerOptionsRustls {
    /// New server Rustls options.
    pub fn new(crt_pem: String, key_pem: String, client_pem: Option<String>) -> Self {
        Self {
            crt_pem,
            key_pem,
            client_pem,
        }
    }
}

/// Server SMTP options.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Server options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOptions {
    hostname: String,
    bind: String,
    /// Enable Pwned Passwords API to check passwords.
    /// API keys may be required in the future to use this API.
    password_pwned_enabled: bool,
    /// Access token expiry time in seconds.
    access_token_expires: i64,
    /// Refresh token expiry time in seconds.
    refresh_token_expires: i64,
    /// Revoke token expiry time in seconds.
    revoke_token_expires: i64,
    /// Authentication provider groups.
    provider: ServerOptionsProviderGroup,
    /// Rustls options for TLS support.
    rustls: Option<ServerOptionsRustls>,
    /// User agent for outgoing HTTP requests.
    user_agent: String,
    /// SMTP transport options.
    smtp_transport: Option<ServerOptionsSmtp>,
    /// SMTP file transport path.
    smtp_file_transport: Option<String>,
}

impl ServerOptions {
    /// Create default options with bind address.
    pub fn new<B>(bind: B) -> Self
    where
        B: Into<String>,
    {
        Self {
            hostname: crate_name!().to_string(),
            bind: bind.into(),
            password_pwned_enabled: false,
            access_token_expires: 3_600,
            refresh_token_expires: 86_400,
            revoke_token_expires: 604_800,
            provider: ServerOptionsProviderGroup::default(),
            rustls: None,
            user_agent: crate_name!().to_string(),
            smtp_transport: None,
            smtp_file_transport: None,
        }
    }

    /// Set hostname.
    pub fn set_hostname<H>(mut self, hostname: H) -> Self
    where
        H: Into<String>,
    {
        self.hostname = hostname.into();
        self
    }

    /// Set password pwned enabled.
    pub fn set_password_pwned_enabled(mut self, password_pwned_enabled: bool) -> Self {
        self.password_pwned_enabled = password_pwned_enabled;
        self
    }

    /// Set provider.
    pub fn set_provider(mut self, provider: ServerOptionsProviderGroup) -> Self {
        self.provider = provider;
        self
    }

    /// Set Rustls options.
    pub fn set_rustls(mut self, rustls: Option<ServerOptionsRustls>) -> Self {
        self.rustls = rustls;
        self
    }

    /// Set user_agent.
    pub fn set_user_agent<UA>(mut self, user_agent: UA) -> Self
    where
        UA: Into<String>,
    {
        self.user_agent = user_agent.into();
        self
    }

    /// Set SMTP transport options.
    pub fn set_smtp_transport(mut self, smtp_transport: Option<ServerOptionsSmtp>) -> Self {
        self.smtp_transport = smtp_transport;
        self
    }

    /// Set SMTP file transport.
    pub fn set_smtp_file_transport(mut self, smtp_file_transport: Option<String>) -> Self {
        self.smtp_file_transport = smtp_file_transport;
        self
    }

    /// Returns hostname reference.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// Returns bind address reference.
    pub fn bind(&self) -> &str {
        &self.bind
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

    /// Returns provider GitHub OAuth2 reference.
    pub fn provider_github_oauth2(&self) -> Option<&AuthProviderOauth2> {
        self.provider.github.oauth2.as_ref()
    }

    /// Returns provider GitHub OAuth2 common arguments.
    pub fn provider_github_oauth2_args(&self) -> AuthProviderOauth2Args {
        AuthProviderOauth2Args::new(
            self.provider_github_oauth2(),
            self.access_token_expires(),
            self.refresh_token_expires(),
        )
    }

    /// Returns provider Microsoft OAuth2 reference.
    pub fn provider_microsoft_oauth2(&self) -> Option<&AuthProviderOauth2> {
        self.provider.microsoft.oauth2.as_ref()
    }

    /// Returns provider Microsoft OAuth2 common arguments.
    pub fn provider_microsoft_oauth2_args(&self) -> AuthProviderOauth2Args {
        AuthProviderOauth2Args::new(
            self.provider_microsoft_oauth2(),
            self.access_token_expires(),
            self.refresh_token_expires(),
        )
    }

    /// Returns rustls server configuration built from options.
    pub fn rustls_server_config(
        options: Option<&ServerOptionsRustls>,
    ) -> DriverResult<Option<ServerConfig>> {
        if let Some(rustls_options) = options {
            let crt_file = File::open(&rustls_options.crt_pem).map_err(DriverError::StdIo)?;
            let key_file = File::open(&rustls_options.key_pem).map_err(DriverError::StdIo)?;
            let crt_file_reader = &mut BufReader::new(crt_file);
            let key_file_reader = &mut BufReader::new(key_file);

            let cert_chain = certs(crt_file_reader).map_err(|_err| DriverError::Rustls)?;
            let mut keys = rsa_private_keys(key_file_reader).map_err(|_err| DriverError::Rustls)?;

            let mut config = if let Some(client_pem) = &rustls_options.client_pem {
                let client_file = File::open(client_pem).map_err(DriverError::StdIo)?;
                let client_file_reader = &mut BufReader::new(client_file);

                let mut roots = RootCertStore::empty();
                roots
                    .add_pem_file(client_file_reader)
                    .map_err(|_err| DriverError::Rustls)?;
                ServerConfig::new(AllowAnyAuthenticatedClient::new(roots))
            } else {
                ServerConfig::new(NoClientAuth::new())
            };
            config.set_single_cert(cert_chain, keys.remove(0)).unwrap();

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    /// Returns user agent.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Returns SMTP client built from options.
    pub fn smtp_client(smtp: Option<&ServerOptionsSmtp>) -> DriverResult<Option<SmtpClient>> {
        if let Some(smtp) = smtp {
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
}

/// Server data.
#[derive(Clone)]
struct Data {
    driver: Box<dyn Driver>,
    options: ServerOptions,
    client: AsyncClient,
    client_sync: SyncClient,
    smtp_client: Option<SmtpClient>,
}

impl Data {
    /// Create new data.
    pub fn new(
        driver: Box<dyn Driver>,
        options: ServerOptions,
        client: AsyncClient,
        client_sync: SyncClient,
        smtp_client: Option<SmtpClient>,
    ) -> Self {
        Data {
            driver,
            options,
            client,
            client_sync,
            smtp_client,
        }
    }

    /// Get reference to driver.
    pub fn driver(&self) -> &dyn Driver {
        self.driver.as_ref()
    }

    /// Get reference to options.
    pub fn options(&self) -> &ServerOptions {
        &self.options
    }

    /// Get reference to asynchronous client.
    pub fn client(&self) -> &AsyncClient {
        &self.client
    }

    /// Get reference to synchronous client.
    pub fn client_sync(&self) -> &SyncClient {
        &self.client_sync
    }

    /// Build email callback function.
    /// If client is None and file directory path is provided, file transport is used.
    pub fn smtp_email(&self) -> Box<dyn FnOnce(TemplateEmail) -> DriverResult<()>> {
        let client = self.smtp_client.clone();
        let from_email = self
            .options()
            .smtp_transport
            .as_ref()
            .map(|x| x.user.to_owned());
        let smtp_file = self
            .options()
            .smtp_file_transport
            .as_ref()
            .map(|x| x.to_owned());

        Box::new(move |email| {
            let email_builder = Email::builder()
                .to((email.to_email, email.to_name))
                .subject(email.subject)
                .text(email.text);

            match (client, smtp_file) {
                (Some(client), _) => {
                    let email = email_builder
                        .from((from_email.unwrap(), email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let mut transport = client.transport();
                    transport.send(email.into()).map_err(DriverError::Lettre)?;
                    Ok(())
                }
                (_, Some(smtp_file)) => {
                    let email = email_builder
                        .from(("file@localhost", email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let path = PathBuf::from(smtp_file);
                    let mut transport = FileTransport::new(path);
                    transport
                        .send(email.into())
                        .map_err(DriverError::LettreFile)?;
                    Ok(())
                }
                (None, None) => Err(DriverError::SmtpDisabled),
            }
        })
    }
}

/// Server functions.
#[derive(Debug)]
pub struct Server;

impl Server {
    /// Start HTTP server.
    pub fn start(
        workers: usize,
        driver: Box<dyn Driver>,
        options: ServerOptions,
    ) -> DriverResult<()> {
        let options_clone = options.clone();
        let client = Self::client_build(options.user_agent())?;
        let client_sync = Self::client_sync_build(options.user_agent())?;
        let smtp_client = ServerOptions::smtp_client(options.smtp_transport.as_ref())?;
        let default_json_limit: usize = 1024;
        let (http_count, http_latency) = Metrics::http_metrics();

        let server = HttpServer::new(move || {
            App::new()
                // Shared data.
                .data(Data::new(
                    driver.clone(),
                    options_clone.clone(),
                    client.clone(),
                    client_sync.clone(),
                    smtp_client.clone(),
                ))
                // Global JSON configuration.
                .data(web::JsonConfig::default().limit(default_json_limit))
                // Header identity middleware.
                .wrap(actix_web_middleware::HeaderIdentityPolicy::identity_service())
                // Metrics middleware.
                .wrap(actix_web_middleware::Metrics::new(
                    http_count.clone(),
                    http_latency.clone(),
                ))
                // Logger middleware.
                .wrap(Logger::default())
                // Route service.
                .configure(route::route_service)
                // Default route (method not allowed).
                .default_service(web::route().to(HttpResponse::MethodNotAllowed))
        })
        .workers(workers)
        .server_hostname(options.hostname());

        let rustls_server_config = ServerOptions::rustls_server_config(options.rustls.as_ref())?;
        let server = if let Some(rustls_server_config) = rustls_server_config {
            server.bind_rustls(options.bind(), rustls_server_config)
        } else {
            server.bind(options.bind())
        }
        .map_err(DriverError::StdIo)?;

        server.start();
        Ok(())
    }

    fn client_build(user_agent: &str) -> DriverResult<AsyncClient> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, user_agent.parse().unwrap());
        AsyncClient::builder()
            .use_rustls_tls()
            .default_headers(headers)
            .build()
            .map_err(DriverError::Reqwest)
    }

    fn client_sync_build(user_agent: &str) -> DriverResult<SyncClient> {
        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, user_agent.parse().unwrap());
        SyncClient::builder()
            .use_rustls_tls()
            .default_headers(headers)
            .build()
            .map_err(DriverError::Reqwest)
    }
}
