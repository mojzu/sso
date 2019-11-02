pub mod actix_web_middleware;
mod route;

use crate::{
    api::{AuthProviderOauth2, AuthProviderOauth2Args},
    Client, Driver, DriverError, DriverResult, Metrics, TemplateEmail,
};
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use lettre::{
    smtp::authentication::{Credentials, Mechanism},
    ClientSecurity, ClientTlsParameters, SmtpClient, Transport,
};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
use reqwest::r#async::Client as ReqwestClient;
use rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    AllowAnyAuthenticatedClient, NoClientAuth, RootCertStore, ServerConfig,
};
use serde::Serialize;
use std::{fs::File, io::BufReader};

// TODO(feature): User sessions route for active tokens/keys.
// TODO(feature): Support more OAuth2 providers.
// TODO(feature): Webauthn support.
// <https://webauthn.guide/>
// <https://webauthn.org/>
// TODO(feature): Configurable canary routes.
// TODO(feature): Improved public library API interface (gui service as example?).
// TODO(feature): Email translation/formatting using user locale and timezone.
// TODO(feature): Handle changes to password hash version.
// TODO(feature): Option to enforce provider URLs HTTPS.
// TODO(feature): User last login, key last use information (calculate in SQL).
// TODO(feature): Login from unknown IP address warnings, SMS support?
// TODO(feature): Jsonwebtoken handling improvements.
// <https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_Cheat_Sheet_for_Java.html>

/// Server options provider options.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServerOptionsProvider {
    oauth2: Option<AuthProviderOauth2>,
}

impl ServerOptionsProvider {
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
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct ServerOptions {
    #[builder(default = "crate_name!().to_string()")]
    hostname: String,
    bind: String,
    /// Enable Pwned Passwords API to check passwords.
    /// API keys may be required in the future to use this API.
    #[builder(default = "false")]
    password_pwned_enabled: bool,
    /// Access token expiry time in seconds.
    #[builder(default = "3_600")]
    access_token_expires: i64,
    /// Refresh token expiry time in seconds.
    #[builder(default = "86_400")]
    refresh_token_expires: i64,
    /// Revoke token expiry time in seconds.
    #[builder(default = "604_800")]
    revoke_token_expires: i64,
    /// Authentication provider groups.
    #[builder(default)]
    provider: ServerOptionsProviderGroup,
    /// Rustls options for TLS support.
    rustls: Option<ServerOptionsRustls>,
    /// User agent for outgoing HTTP requests.
    #[builder(default = "crate_name!().to_string()")]
    user_agent: String,
    /// SMTP options.
    smtp: Option<ServerOptionsSmtp>,
}

impl ServerOptions {
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
            self.user_agent(),
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
            self.user_agent(),
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
    pub fn smtp_client(options: Option<&ServerOptionsSmtp>) -> DriverResult<Option<SmtpClient>> {
        if let Some(smtp_options) = options {
            let mut tls_builder = TlsConnector::builder();
            tls_builder.min_protocol_version(Some(Protocol::Tlsv10));
            let tls_parameters = ClientTlsParameters::new(
                smtp_options.host.to_owned(),
                tls_builder.build().map_err(DriverError::NativeTls)?,
            );

            let client = SmtpClient::new(
                (smtp_options.host.as_ref(), smtp_options.port),
                ClientSecurity::Required(tls_parameters),
            )
            .map_err(DriverError::Lettre)?
            .authentication_mechanism(Mechanism::Login)
            .credentials(Credentials::new(
                smtp_options.user.to_owned(),
                smtp_options.password.to_owned(),
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
    client: ReqwestClient,
    smtp_client: Option<SmtpClient>,
}

impl Data {
    /// Create new data.
    pub fn new(
        driver: Box<dyn Driver>,
        options: ServerOptions,
        client: ReqwestClient,
        smtp_client: Option<SmtpClient>,
    ) -> Self {
        Data {
            driver,
            options,
            client,
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
    pub fn client(&self) -> &ReqwestClient {
        &self.client
    }

    /// Build email callback function.
    pub fn smtp_email(&self) -> Box<dyn FnOnce(TemplateEmail) -> DriverResult<()>> {
        let client = self.smtp_client.clone();
        let from_email = self.options().smtp.as_ref().map(|x| x.user.to_owned());
        Box::new(move |email| match client {
            Some(client) => {
                let email = Email::builder()
                    .to((email.to_email, email.to_name))
                    .from((from_email.unwrap(), email.from_name))
                    .subject(email.subject)
                    .text(email.text)
                    .build()
                    .map_err(DriverError::LettreEmail)?;
                let mut transport = client.transport();

                transport.send(email.into()).map_err(DriverError::Lettre)?;
                Ok(())
            }
            None => Err(DriverError::SmtpDisabled),
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
        let client = Client::build_client(options.user_agent())?;
        let smtp_client = ServerOptions::smtp_client(options.smtp.as_ref())?;
        let default_json_limit: usize = 1024;
        let (http_count, http_latency) = Metrics::http_metrics();

        let server = HttpServer::new(move || {
            App::new()
                // Shared data.
                .data(Data::new(
                    driver.clone(),
                    options_clone.clone(),
                    client.clone(),
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
}
