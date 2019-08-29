//! # Server
pub mod api;
mod error;
pub mod middleware;
mod route;
pub mod validate;

pub use crate::server::error::{Error as ServerError, Oauth2Error as ServerOauth2Error};
pub use crate::server::validate::FromJsonValue;

use crate::client::ClientActor;
use crate::notify::NotifyActor;
use crate::server::error::{Error, Oauth2Error};
use crate::{core, driver::Driver};
use actix::Addr;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer};
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{AllowAnyAuthenticatedClient, NoClientAuth, RootCertStore, ServerConfig};
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;

// TODO(feature): User sessions route for active tokens/keys.
// TODO(feature): Better method to handle multiple keys?
// Allow or require specifying key ID via argument?
// TODO(feature): Support more OAuth2 providers.
// TODO(feature): Webauthn support.
// <https://webauthn.guide/>
// <https://webauthn.org/>
// TODO(feature): Configurable canary routes.
// TODO(feature): Improved public library API interface.
// TODO(feature): All emails have 2 actions, ok or revoke, option to verify update email/password requests.
// TODO(feature): Email translation/formatting using user locale and timezone.

/// Default JSON payload size limit.
const DEFAULT_JSON_LIMIT: usize = 1024;

/// Provider OAuth2 options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerOptionsProviderOauth2 {
    client_id: String,
    client_secret: String,
    redirect_url: String,
}

impl ServerOptionsProviderOauth2 {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
        }
    }
}

/// Provider options.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ServerOptionsProvider {
    oauth2: Option<ServerOptionsProviderOauth2>,
}

impl ServerOptionsProvider {
    pub fn new(oauth2: Option<ServerOptionsProviderOauth2>) -> Self {
        Self { oauth2 }
    }
}

// Provider group options.
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

// Rustls options.
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
    pub fn provider_github_oauth2(&self) -> Option<&ServerOptionsProviderOauth2> {
        self.provider.github.oauth2.as_ref()
    }

    /// Returns provider Microsoft OAuth2 reference.
    pub fn provider_microsoft_oauth2(&self) -> Option<&ServerOptionsProviderOauth2> {
        self.provider.microsoft.oauth2.as_ref()
    }

    /// Returns rustls server configuration built from options.
    pub fn rustls_server_config(
        options: Option<&ServerOptionsRustls>,
    ) -> Result<Option<ServerConfig>, Error> {
        if let Some(rustls_options) = options {
            let crt_file = File::open(&rustls_options.crt_pem).map_err(Error::StdIo)?;
            let key_file = File::open(&rustls_options.key_pem).map_err(Error::StdIo)?;
            let crt_file_reader = &mut BufReader::new(crt_file);
            let key_file_reader = &mut BufReader::new(key_file);

            let cert_chain = certs(crt_file_reader).map_err(|_err| Error::Rustls)?;
            let mut keys = rsa_private_keys(key_file_reader).map_err(|_err| Error::Rustls)?;

            let mut config = if let Some(client_pem) = &rustls_options.client_pem {
                let client_file = File::open(client_pem).map_err(Error::StdIo)?;
                let client_file_reader = &mut BufReader::new(client_file);

                let mut roots = RootCertStore::empty();
                roots
                    .add_pem_file(client_file_reader)
                    .map_err(|_err| Error::Rustls)?;
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
pub struct Data {
    driver: Box<dyn Driver>,
    options: ServerOptions,
    notify_addr: Addr<NotifyActor>,
    client_addr: Addr<ClientActor>,
    registry: Registry,
}

impl Data {
    /// Create new data.
    pub fn new(
        driver: Box<dyn Driver>,
        options: ServerOptions,
        notify_addr: Addr<NotifyActor>,
        client_addr: Addr<ClientActor>,
        registry: Registry,
    ) -> Self {
        Data {
            driver,
            options,
            notify_addr,
            client_addr,
            registry,
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

    /// Get reference to notify actor address.
    pub fn notify(&self) -> &Addr<NotifyActor> {
        &self.notify_addr
    }

    /// Get reference to client actor address.
    pub fn client(&self) -> &Addr<ClientActor> {
        &self.client_addr
    }

    /// Get reference to prometheus registry.
    pub fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }
}

/// Server unit struct.
pub struct Server;

impl Server {
    /// Start HTTP server.
    pub fn start(
        workers: usize,
        driver: Box<dyn Driver>,
        options: ServerOptions,
        notify_addr: Addr<NotifyActor>,
        client_addr: Addr<ClientActor>,
    ) -> Result<(), Error> {
        let options_clone = options.clone();
        let (registry, counter, histogram) = metrics_registry()?;

        let server = HttpServer::new(move || {
            App::new()
                // Shared data.
                .data(Data::new(
                    driver.clone(),
                    options_clone.clone(),
                    notify_addr.clone(),
                    client_addr.clone(),
                    registry.clone(),
                ))
                // Global JSON configuration.
                .data(web::JsonConfig::default().limit(DEFAULT_JSON_LIMIT))
                // Authorisation header identity middleware.
                .wrap(middleware::AuthorisationIdentityPolicy::identity_service())
                // Metrics middleware.
                .wrap(middleware::Metrics::new(counter.clone(), histogram.clone()))
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
        .map_err(Error::StdIo)?;

        server.start();
        Ok(())
    }
}

fn metrics_registry() -> Result<(Registry, IntCounterVec, HistogramVec), Error> {
    let registry = Registry::new();
    let count_opts = Opts::new(
        core::metrics::name("http_count"),
        "HTTP request counter".to_owned(),
    );
    let count = IntCounterVec::new(count_opts, &["path", "status"]).map_err(Error::Prometheus)?;

    let latency_opts = HistogramOpts::new(
        core::metrics::name("http_latency"),
        "HTTP request latency".to_owned(),
    );
    let latency = HistogramVec::new(latency_opts, &["path"]).map_err(Error::Prometheus)?;

    registry
        .register(Box::new(count.clone()))
        .map_err(Error::Prometheus)?;
    registry
        .register(Box::new(latency.clone()))
        .map_err(Error::Prometheus)?;
    Ok((registry, count, latency))
}
