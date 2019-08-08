//! # Server
pub mod api;
pub mod metrics;
pub mod route;
pub mod validate;

use crate::client::ClientExecutor;
use crate::notify::NotifyExecutor;
use crate::{core, driver};
use actix::Addr;
use actix_identity::{IdentityPolicy, IdentityService};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry};
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{AllowAnyAuthenticatedClient, NoClientAuth, RootCertStore, ServerConfig};
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
pub use validate::FromJsonValue;

// TODO(feature): User sessions route for active tokens/keys.
// TODO(feature): Better method to handle multiple keys?
// Allow or require specifying key ID via argument?
// TODO(feature): Support more OAuth2 providers.
// TODO(feature): Webauthn support.
// <https://webauthn.guide/>
// <https://webauthn.org/>
// TODO(feature): Configurable canary routes.
// TODO(feature): Improved public library API, function interfaces, use of Into?
// TODO(feature): All emails have 2 actions, ok or revoke, option to verify update email/password requests.
// TODO(feature): Email translation/formatting using user locale and timezone.

/// Default JSON payload size limit.
const DEFAULT_JSON_LIMIT: usize = 1024;

/// Pwned passwords errors.
#[derive(Debug, Fail)]
pub enum PwnedPasswordsError {
    /// Integration disabled.
    #[fail(display = "PwnedPasswordsError::Disabled")]
    Disabled,
    /// Status code error.
    #[fail(display = "PwnedPasswordsError::StatusCode {}", _0)]
    StatusCode(actix_web::http::StatusCode),
    /// From UTF8 string error.
    #[fail(display = "PwnedPasswordsError::FromUtf8 {}", _0)]
    FromUtf8(std::string::FromUtf8Error),
    /// Client send request error.
    #[fail(display = "PwnedPasswordsError::ActixClientSendRequest")]
    ActixClientSendRequest,
    /// Payload error.
    #[fail(display = "PwnedPasswordsError::ActixPayload")]
    ActixPayload,
}

/// OAuth2 errors.
#[derive(Debug, Fail)]
pub enum Oauth2Error {
    /// Integration disabled.
    #[fail(display = "Oauth2Error::Disabled")]
    Disabled,
    /// CSRF error.
    #[fail(display = "Oauth2Error::Csrf")]
    Csrf,
    /// Status code error.
    #[fail(display = "Oauth2Error::StatusCode {}", _0)]
    StatusCode(actix_web::http::StatusCode),
    /// OAuth2 request token error.
    #[fail(display = "Oauth2Error::Oauth2Request {}", _0)]
    Oauth2Request(failure::Error),
    /// Client send request error.
    #[fail(display = "Oauth2Error::ActixClientSendRequest")]
    ActixClientSendRequest,
    /// Payload error.
    #[fail(display = "Oauth2Error::ActixPayload")]
    ActixPayload,
}

/// Server errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Bad request, deserialisation failure.
    #[fail(display = "ServerError::BadRequest")]
    BadRequest,
    /// Forbidden, authentication failure.
    #[fail(display = "ServerError::Forbidden")]
    Forbidden,
    /// Not found.
    #[fail(display = "ServerError::NotFound")]
    NotFound,
    /// Client request error.
    #[fail(display = "ServerError::PwnedPasswords {}", _0)]
    PwnedPasswords(PwnedPasswordsError),
    /// OAuth2 error.
    #[fail(display = "ServerError::Oauth2 {}", _0)]
    Oauth2(Oauth2Error),
    /// Core error wrapper.
    #[fail(display = "ServerError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Rustls error wrapper.
    #[fail(display = "ServerError::Rustls")]
    Rustls,
    /// URL parse error.
    #[fail(display = "ServerError::UrlParse {}", _0)]
    UrlParse(#[fail(cause)] url::ParseError),
    /// Actix web blocking error cancelled wrapper.
    #[fail(display = "ServerError::ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,
    /// Standard IO error wrapper.
    #[fail(display = "ServerError::StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
    /// Zxcvbn error wrapper.
    #[fail(display = "ServerError::Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] zxcvbn::ZxcvbnError),
    /// Handlebars template render error wrapper.
    #[fail(display = "ServerError::HandlebarsTemplateRender {}", _0)]
    HandlebarsTemplateRender(#[fail(cause)] handlebars::TemplateRenderError),
    /// Prometheus error wrapper.
    #[fail(display = "ServerError::Prometheus {}", _0)]
    Prometheus(#[fail(cause)] prometheus::Error),
}

impl From<core::Error> for Error {
    fn from(err: core::Error) -> Self {
        match err {
            core::Error::BadRequest => Error::BadRequest,
            core::Error::Forbidden => Error::Forbidden,
            core::Error::Jsonwebtoken(_e) => Error::BadRequest,
            _ => Error::Core(err),
        }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::BadRequest => HttpResponse::BadRequest().finish(),
            Error::Forbidden => HttpResponse::Forbidden().finish(),
            Error::NotFound => HttpResponse::NotFound().finish(),
            _ => {
                error!("{}", self);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

impl From<actix_web::error::BlockingError<Error>> for Error {
    fn from(e: actix_web::error::BlockingError<Error>) -> Self {
        match e {
            actix_web::error::BlockingError::Error(e) => e,
            actix_web::error::BlockingError::Canceled => Error::ActixWebBlockingCancelled,
        }
    }
}

/// Provider OAuth2 configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProviderOauth2 {
    client_id: String,
    client_secret: String,
    redirect_url: String,
}

impl ConfigurationProviderOauth2 {
    pub fn new(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
        }
    }
}

/// Provider configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigurationProvider {
    oauth2: Option<ConfigurationProviderOauth2>,
}

impl ConfigurationProvider {
    pub fn new(oauth2: Option<ConfigurationProviderOauth2>) -> Self {
        Self { oauth2 }
    }
}

// Provider group configuration.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ConfigurationProviderGroup {
    github: ConfigurationProvider,
    microsoft: ConfigurationProvider,
}

impl ConfigurationProviderGroup {
    pub fn new(github: ConfigurationProvider, microsoft: ConfigurationProvider) -> Self {
        Self { github, microsoft }
    }
}

// Rustls configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationRustls {
    crt_pem: String,
    key_pem: String,
    client_pem: Option<String>,
}

impl ConfigurationRustls {
    pub fn new(crt_pem: String, key_pem: String, client_pem: Option<String>) -> Self {
        Self {
            crt_pem,
            key_pem,
            client_pem,
        }
    }
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
pub struct Configuration {
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
    provider: ConfigurationProviderGroup,
    /// Rustls configuration for TLS support.
    rustls: Option<ConfigurationRustls>,
}

impl Configuration {
    /// Configured hostname.
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    /// Configured bind address.
    pub fn bind(&self) -> &str {
        &self.bind
    }

    /// Get password pwned enabled.
    pub fn password_pwned_enabled(&self) -> bool {
        self.password_pwned_enabled
    }

    /// Get access token expiry.
    pub fn access_token_expires(&self) -> i64 {
        self.access_token_expires
    }

    /// Get refresh token expiry.
    pub fn refresh_token_expires(&self) -> i64 {
        self.refresh_token_expires
    }

    /// Get revoke token expiry.
    pub fn revoke_token_expires(&self) -> i64 {
        self.revoke_token_expires
    }

    /// Configured provider GitHub OAuth2.
    pub fn provider_github_oauth2(&self) -> Option<&ConfigurationProviderOauth2> {
        self.provider.github.oauth2.as_ref()
    }

    /// Configured provider Microsoft OAuth2.
    pub fn provider_microsoft_oauth2(&self) -> Option<&ConfigurationProviderOauth2> {
        self.provider.microsoft.oauth2.as_ref()
    }

    /// Configured rustls server parameters.
    pub fn rustls_server_config(&self) -> Result<Option<ServerConfig>, Error> {
        if let Some(rustls_config) = &self.rustls {
            let crt_file = File::open(&rustls_config.crt_pem).map_err(Error::StdIo)?;
            let key_file = File::open(&rustls_config.key_pem).map_err(Error::StdIo)?;
            let crt_file_reader = &mut BufReader::new(crt_file);
            let key_file_reader = &mut BufReader::new(key_file);

            let cert_chain = certs(crt_file_reader).map_err(|_err| Error::Rustls)?;
            let mut keys = rsa_private_keys(key_file_reader).map_err(|_err| Error::Rustls)?;

            let mut config = if let Some(client_pem) = &rustls_config.client_pem {
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
    driver: Box<driver::Driver>,
    configuration: Configuration,
    notify_addr: Addr<NotifyExecutor>,
    client_addr: Addr<ClientExecutor>,
    registry: Registry,
}

impl Data {
    /// Create new data.
    pub fn new(
        driver: Box<driver::Driver>,
        configuration: Configuration,
        notify_addr: Addr<NotifyExecutor>,
        client_addr: Addr<ClientExecutor>,
        registry: Registry,
    ) -> Self {
        Data {
            driver,
            configuration,
            notify_addr,
            client_addr,
            registry,
        }
    }

    /// Get reference to driver.
    pub fn driver(&self) -> &driver::Driver {
        self.driver.as_ref()
    }

    /// Get reference to configuration.
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    /// Get reference to notify actor address.
    pub fn notify(&self) -> &Addr<NotifyExecutor> {
        &self.notify_addr
    }

    /// Get reference to prometheus registry.
    pub fn registry(&self) -> &prometheus::Registry {
        &self.registry
    }
}

/// Authorisation identity policy.
pub struct AuthorisationIdentityPolicy {
    header: String,
}

impl AuthorisationIdentityPolicy {
    /// Create new identity service.
    pub fn identity_service() -> IdentityService<Self> {
        IdentityService::new(AuthorisationIdentityPolicy::default())
    }
}

impl Default for AuthorisationIdentityPolicy {
    fn default() -> Self {
        AuthorisationIdentityPolicy {
            header: "Authorization".to_owned(),
        }
    }
}

impl IdentityPolicy for AuthorisationIdentityPolicy {
    type Future = actix_web::Result<Option<String>, actix_web::Error>;
    type ResponseFuture = actix_web::Result<(), actix_web::Error>;

    fn from_request(&self, request: &mut ServiceRequest) -> Self::Future {
        let key = match request.headers().get(&self.header) {
            Some(value) => {
                let value = value.to_str().map_err(|_err| Error::Forbidden)?;
                trim_authorisation(value)
            }
            None => None,
        };
        Ok(key)
    }

    fn to_response<B>(
        &self,
        _id: Option<String>,
        _changed: bool,
        _response: &mut ServiceResponse<B>,
    ) -> Self::ResponseFuture {
        Ok(())
    }
}

/// Returns key value from formats: `$KEY`, `Bearer $KEY`.
fn trim_authorisation(value: &str) -> Option<String> {
    let value = value.to_owned();
    if value.starts_with("Bearer ") {
        let parts: Vec<&str> = value.split(' ').collect();
        if parts.len() > 1 {
            let value = parts[1].trim().to_owned();
            Some(value)
        } else {
            None
        }
    } else {
        Some(value)
    }
}

/// Start HTTP server.
pub fn start(
    workers: usize,
    driver: Box<driver::Driver>,
    configuration: Configuration,
    notify_addr: Addr<NotifyExecutor>,
    client_addr: Addr<ClientExecutor>,
) -> Result<(), Error> {
    let configuration_clone = configuration.clone();
    let (registry, counter, histogram) = metrics_registry()?;

    let server = HttpServer::new(move || {
        App::new()
            // Shared data.
            .data(Data::new(
                driver.clone(),
                configuration_clone.clone(),
                notify_addr.clone(),
                client_addr.clone(),
                registry.clone(),
            ))
            // Global JSON configuration.
            .data(web::JsonConfig::default().limit(DEFAULT_JSON_LIMIT))
            // Authorisation header identity middleware.
            .wrap(AuthorisationIdentityPolicy::identity_service())
            // Metrics middleware.
            .wrap(metrics::Metrics::new(counter.clone(), histogram.clone()))
            // Logger middleware.
            .wrap(middleware::Logger::default())
            // Route service.
            .configure(route::route_service)
            // Default route (method not allowed).
            .default_service(web::route().to(HttpResponse::MethodNotAllowed))
    })
    .workers(workers)
    .server_hostname(configuration.hostname());

    let rustls_server_config = configuration.rustls_server_config()?;
    let server = if let Some(rustls_server_config) = rustls_server_config {
        server.bind_rustls(configuration.bind(), rustls_server_config)
    } else {
        server.bind(configuration.bind())
    }
    .map_err(Error::StdIo)?;

    server.start();
    Ok(())
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
