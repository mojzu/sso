//! # Server
//! HTTP server.
pub mod auth;
pub mod key;
pub mod service;
pub mod user;

use crate::{core, driver};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware,
    middleware::identity::{IdentityPolicy, IdentityService},
    web, App, HttpResponse, HttpServer, ResponseError,
};
use futures::{future, Future};
use serde::de::DeserializeOwned;
use serde::Serialize;
use validator::{Validate, ValidationError};

// TODO(feature): Audit logging, x-forwarded-for header.
// TODO(feature): Prometheus metrics.
// <https://prometheus.io/docs/instrumenting/exposition_formats/>

/// Server errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Bad request, deserialisation failure.
    #[fail(display = "ServerError::BadRequest")]
    BadRequest,
    /// Forbidden, authentication failure.
    #[fail(display = "ServerError::Forbidden")]
    Forbidden,
    /// Client request error.
    #[fail(display = "ServerError::ApiPwnedPasswords")]
    ApiPwnedPasswords,
    /// Core error wrapper.
    #[fail(display = "ServerError::CoreError {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Actix web blocking error cancelled wrapper.
    #[fail(display = "ServerError::ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,
    /// Standard IO error wrapper.
    #[fail(display = "ServerError::StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
    /// Zxcvbn error wrapper.
    #[fail(display = "ServerError::Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] zxcvbn::ZxcvbnError),
}

impl From<core::Error> for Error {
    fn from(error: core::Error) -> Self {
        Error::Core(error)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            // TODO(refactor): Refactor this.
            // ApiError::BadRequest => HttpResponse::BadRequest().finish(),
            // ApiError::Forbidden => HttpResponse::Forbidden().finish(),
            // ApiError::InvalidOauth2Provider => HttpResponse::MethodNotAllowed().finish(),
            // ApiError::Db(e) => {
            //     error!("{}", e);
            //     HttpResponse::InternalServerError().finish()
            // }
            _e => {
                error!("{}", _e);
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

/// OAuth2 provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationOauth2Provider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
}

/// OAuth2 configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationOauth2 {
    github: Option<ConfigurationOauth2Provider>,
    microsoft: Option<ConfigurationOauth2Provider>,
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationSmtp {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    bind: String,
    user_agent: String,
    password_pwned_enabled: bool,
    smtp: Option<ConfigurationSmtp>,
    oauth2: ConfigurationOauth2,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(bind: String) -> Self {
        Configuration {
            bind,
            user_agent: Configuration::default_user_agent(),
            password_pwned_enabled: false,
            smtp: None,
            oauth2: ConfigurationOauth2 {
                github: None,
                microsoft: None,
            },
        }
    }

    /// Set password pwned enabled.
    pub fn set_password_pwned_enabled(mut self, value: bool) -> Self {
        self.password_pwned_enabled = value;
        self
    }

    // Set SMTP provider.
    pub fn set_smtp(mut self, host: String, port: u16, user: String, password: String) -> Self {
        self.smtp = Some(ConfigurationSmtp {
            host,
            port,
            user,
            password,
        });
        self
    }

    /// Set GitHub OAuth2 provider.
    pub fn set_oauth2_github(
        mut self,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        self.oauth2.github = Some(ConfigurationOauth2Provider {
            client_id,
            client_secret,
            redirect_url,
        });
        self
    }

    /// Set Microsoft OAuth2 provider.
    pub fn set_oauth2_microsoft(
        mut self,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        self.oauth2.microsoft = Some(ConfigurationOauth2Provider {
            client_id,
            client_secret,
            redirect_url,
        });
        self
    }

    /// Configured bind address.
    pub fn bind(&self) -> &str {
        &self.bind
    }

    /// Configured user agent.
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }

    /// Get password pwned enabled.
    pub fn password_pwned_enabled(&self) -> bool {
        self.password_pwned_enabled == true
    }

    /// Configured SMTP provider.
    pub fn smtp(&self) -> Option<&ConfigurationSmtp> {
        self.smtp.as_ref()
    }

    /// Configured GitHub OAuth2 provider.
    pub fn oauth2_github(&self) -> Option<&ConfigurationOauth2Provider> {
        self.oauth2.github.as_ref()
    }

    /// Configured Microsoft OAuth2 provider.
    pub fn oauth2_microsoft(&self) -> Option<&ConfigurationOauth2Provider> {
        self.oauth2.microsoft.as_ref()
    }

    /// Default user agent constructed from crate name and version.
    fn default_user_agent() -> String {
        format!("{}/{}", crate_name!(), crate_version!())
    }
}

/// Server data.
#[derive(Clone)]
pub struct Data {
    configuration: Configuration,
    driver: Box<driver::Driver>,
}

impl Data {
    /// Create new data.
    pub fn new(configuration: Configuration, driver: Box<driver::Driver>) -> Self {
        Data {
            configuration,
            driver,
        }
    }

    /// Get reference to configuration.
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }

    /// Get reference to driver.
    pub fn driver(&self) -> &driver::Driver {
        self.driver.as_ref()
    }
}

/// Authorisation identity policy.
pub struct AuthorisationIdentityPolicy {
    header: String,
}

impl AuthorisationIdentityPolicy {
    /// Create new identity policy.
    pub fn new() -> Self {
        AuthorisationIdentityPolicy {
            header: "Authorization".to_owned(),
        }
    }

    /// Create new identity service.
    pub fn identity_service() -> IdentityService<Self> {
        IdentityService::new(AuthorisationIdentityPolicy::new())
    }
}

impl IdentityPolicy for AuthorisationIdentityPolicy {
    type Future = actix_web::Result<Option<String>, actix_web::Error>;
    type ResponseFuture = actix_web::Result<(), actix_web::Error>;

    fn from_request(&self, request: &mut ServiceRequest) -> Self::Future {
        let key = match request.headers().get(&self.header) {
            Some(value) => {
                let value = value.to_str().map_err(|_| Error::Forbidden)?;
                Some(value.to_owned())
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

/// API version 1 ping route.
pub fn api_v1_ping() -> actix_web::Result<HttpResponse> {
    let body = r#"pong"#;
    Ok(HttpResponse::Ok().json(body))
}

/// API version 1 service scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/v1")
        .service(web::resource("/ping").route(web::get().to(api_v1_ping)))
        .service(service::api_v1_scope())
        .service(user::api_v1_scope())
}

/// Service configuration.
pub fn api_service(configuration: &mut web::ServiceConfig) {
    configuration.service(api_v1_scope());
}

/// Start HTTP server.
pub fn start(configuration: Configuration, driver: Box<driver::Driver>) -> Result<(), Error> {
    let bind = configuration.bind().to_owned();

    let server = HttpServer::new(move || {
        App::new()
            // Shared data.
            .data(Data::new(configuration.clone(), driver.clone()))
            // Logger middleware.
            .wrap(middleware::Logger::default())
            // TODO(refactor): Sentry middleware support.
            // Authorisation header identity service.
            .wrap(AuthorisationIdentityPolicy::identity_service())
            // API service.
            .configure(api_service)
            // Default route (method not allowed).
            .default_service(web::route().to(HttpResponse::MethodNotAllowed))
    })
    .bind(bind)
    .map_err(Error::StdIo)?;

    server.start();
    Ok(())
}

/// Route JSON size limit configuration.
pub fn route_json_config() -> web::JsonConfig {
    web::JsonConfig::default().limit(1024)
}

/// Route response empty handler.
pub fn route_response_empty<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(_r) => future::ok(HttpResponse::Ok().finish()),
        Err(e) => future::ok(e.error_response()),
    }
}

/// Route response handler.
pub fn route_response_json<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(r) => future::ok(HttpResponse::Ok().json(r)),
        Err(e) => future::ok(e.error_response()),
    }
}

// TODO(refactor)
/// Validate JSON value trait.
pub trait ValidateFromValue<T: DeserializeOwned + Validate> {
    /// Extract and validate data from JSON value.
    fn from_value(value: serde_json::Value) -> future::FutureResult<T, Error> {
        future::result(
            serde_json::from_value::<T>(value)
                .map_err(|_e| Error::BadRequest)
                .and_then(|body| {
                    body.validate().map_err(|_e| Error::BadRequest)?;
                    Ok(body)
                }),
        )
    }
}

pub fn validate_unsigned(id: i64) -> Result<(), ValidationError> {
    if id < 0 {
        Err(ValidationError::new("invalid_unsigned"))
    } else {
        Ok(())
    }
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.is_empty() || password.len() > 100 {
        Err(ValidationError::new("invalid_password"))
    } else {
        Ok(())
    }
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 100 {
        Err(ValidationError::new("invalid_name"))
    } else {
        Ok(())
    }
}

pub fn validate_id(id: i64) -> Result<(), ValidationError> {
    if id < 1 {
        Err(ValidationError::new("invalid_id"))
    } else {
        Ok(())
    }
}

pub fn validate_token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() || token.len() > 1024 {
        Err(ValidationError::new("invalid_token"))
    } else {
        Ok(())
    }
}

pub fn validate_key(key: &str) -> Result<(), ValidationError> {
    if key.is_empty() || key.len() > 32 {
        Err(ValidationError::new("invalid_key"))
    } else {
        Ok(())
    }
}
