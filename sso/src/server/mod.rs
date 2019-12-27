mod route;

// TODO(refactor): Remove deprecated server code.

use crate::{
    api::{AuthProviderOauth2, AuthProviderOauth2Args},
    Driver, DriverError, DriverResult, TemplateEmail,
};
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer};
use http::{header, HeaderMap};
use lettre_email::Email;
use native_tls::{Protocol, TlsConnector};
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
}

/// Server data.
#[derive(Clone)]
struct Data {
    driver: Box<dyn Driver>,
    options: ServerOptions,
}

impl Data {
    /// Create new data.
    pub fn new(
        driver: Box<dyn Driver>,
        options: ServerOptions,
    ) -> Self {
        Data {
            driver,
            options,
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
        let default_json_limit: usize = 1024;

        let server = HttpServer::new(move || {
            App::new()
                // Shared data.
                .data(Data::new(
                    driver.clone(),
                    options_clone.clone(),
                ))
                // Global JSON configuration.
                .data(web::JsonConfig::default().limit(default_json_limit))
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
