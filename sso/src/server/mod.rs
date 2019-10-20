pub mod actix_web_middleware;
mod route;

use crate::{
    api::{ApiError, AuthProviderOauth2, AuthProviderOauth2Args},
    ClientActor, ClientError, CoreError, Driver, Metrics, NotifyActor,
};
use actix::{Addr, MailboxError as ActixMailboxError};
use actix_web::{
    error::BlockingError as ActixWebBlockingError, middleware::Logger, web, App, HttpResponse,
    HttpServer, ResponseError,
};
use prometheus::{
    Error as PrometheusError, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry,
};
use rustls::{
    internal::pemfile::{certs, rsa_private_keys},
    AllowAnyAuthenticatedClient, NoClientAuth, RootCertStore, ServerConfig,
};
use serde::Serialize;
use std::{
    fs::File,
    io::{BufReader, Error as StdIoError},
};

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

/// Server errors.
#[derive(Debug, Fail)]
pub enum ServerError {
    #[fail(display = "ServerError:Client {}", _0)]
    Client(#[fail(cause)] ClientError),

    #[fail(display = "ServerError:Rustls")]
    Rustls,

    #[fail(display = "ServerError:ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,

    #[fail(display = "ServerError:ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] ActixMailboxError),

    #[fail(display = "ServerError:StdIo {}", _0)]
    StdIo(#[fail(cause)] StdIoError),

    #[fail(display = "ServerError:Prometheus {}", _0)]
    Prometheus(#[fail(cause)] PrometheusError),
}

/// Server result wrapper type.
pub type ServerResult<T> = Result<T, ServerError>;

impl From<ActixWebBlockingError<ApiError>> for ApiError {
    fn from(e: ActixWebBlockingError<ApiError>) -> Self {
        match e {
            ActixWebBlockingError::Error(e) => e,
            ActixWebBlockingError::Canceled => {
                Self::InternalServerError(CoreError::ActixWebBlockingCancelled)
            }
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::BadRequest(_e) => HttpResponse::BadRequest().finish(),
            Self::Unauthorised(_e) => HttpResponse::Unauthorized().finish(),
            Self::Forbidden(_e) => HttpResponse::Forbidden().finish(),
            Self::NotFound(_e) => HttpResponse::NotFound().finish(),
            _ => {
                error!("{}", self);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

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
    user_agent: String,
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
    ) -> ServerResult<Option<ServerConfig>> {
        if let Some(rustls_options) = options {
            let crt_file = File::open(&rustls_options.crt_pem).map_err(ServerError::StdIo)?;
            let key_file = File::open(&rustls_options.key_pem).map_err(ServerError::StdIo)?;
            let crt_file_reader = &mut BufReader::new(crt_file);
            let key_file_reader = &mut BufReader::new(key_file);

            let cert_chain = certs(crt_file_reader).map_err(|_err| ServerError::Rustls)?;
            let mut keys = rsa_private_keys(key_file_reader).map_err(|_err| ServerError::Rustls)?;

            let mut config = if let Some(client_pem) = &rustls_options.client_pem {
                let client_file = File::open(client_pem).map_err(ServerError::StdIo)?;
                let client_file_reader = &mut BufReader::new(client_file);

                let mut roots = RootCertStore::empty();
                roots
                    .add_pem_file(client_file_reader)
                    .map_err(|_err| ServerError::Rustls)?;
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
}

/// Server data.
#[derive(Clone)]
struct Data {
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

/// Server functions.
#[derive(Debug)]
pub struct Server;

impl Server {
    /// Start HTTP server.
    pub fn start(
        workers: usize,
        driver: Box<dyn Driver>,
        options: ServerOptions,
        notify_addr: Addr<NotifyActor>,
        client_addr: Addr<ClientActor>,
    ) -> ServerResult<()> {
        let options_clone = options.clone();
        let (registry, counter, histogram) = Server::metrics_registry()?;
        let default_json_limit: usize = 1024;

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
                .data(web::JsonConfig::default().limit(default_json_limit))
                // Authorisation header identity middleware.
                .wrap(actix_web_middleware::AuthorisationIdentityPolicy::identity_service())
                // Metrics middleware.
                .wrap(actix_web_middleware::Metrics::new(
                    counter.clone(),
                    histogram.clone(),
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
        .map_err(ServerError::StdIo)?;

        server.start();
        Ok(())
    }

    fn metrics_registry() -> ServerResult<(Registry, IntCounterVec, HistogramVec)> {
        let registry = Registry::new();
        let count_opts = Opts::new(
            Metrics::name("http_count"),
            "HTTP request counter".to_owned(),
        );
        let count =
            IntCounterVec::new(count_opts, &["path", "status"]).map_err(ServerError::Prometheus)?;

        let latency_opts = HistogramOpts::new(
            Metrics::name("http_latency"),
            "HTTP request latency".to_owned(),
        );
        let latency =
            HistogramVec::new(latency_opts, &["path"]).map_err(ServerError::Prometheus)?;

        registry
            .register(Box::new(count.clone()))
            .map_err(ServerError::Prometheus)?;
        registry
            .register(Box::new(latency.clone()))
            .map_err(ServerError::Prometheus)?;
        Ok((registry, count, latency))
    }
}
