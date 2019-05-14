pub mod route;
pub mod smtp;
pub mod validate;

pub use validate::FromJsonValue;

use crate::{core, driver};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware,
    middleware::identity::{IdentityPolicy, IdentityService},
    web, App, HttpResponse, HttpServer, ResponseError,
};
use futures::{future, Future};
use serde::Serialize;

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
    /// Not found.
    #[fail(display = "ServerError::NotFound")]
    NotFound,
    /// Client request error.
    #[fail(display = "ServerError::ApiPwnedPasswords")]
    ApiPwnedPasswords,
    /// SMTP error.
    #[fail(display = "ServerError::Smtp")]
    Smtp,
    /// OAuth2 error.
    #[fail(display = "ServerError::Oauth2")]
    Oauth2,
    /// Core error wrapper.
    #[fail(display = "ServerError::Core {}", _0)]
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
    fn from(err: core::Error) -> Self {
        match err {
            core::Error::BadRequest => Error::BadRequest,
            core::Error::Forbidden => Error::Forbidden,
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

/// OAuth2 provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationOauth2Provider {
    client_id: String,
    client_secret: String,
    redirect_url: String,
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
    host: String,
    port: u16,
    user: String,
    password: String,
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
        self.password_pwned_enabled
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

fn ping_handler() -> actix_web::Result<HttpResponse> {
    let body = r#"pong"#;
    Ok(HttpResponse::Ok().json(body))
}

/// API version 1 service scope.
pub fn api_v1_scope() -> actix_web::Scope {
    web::scope("/v1")
        .service(web::resource("/ping").route(web::get().to(ping_handler)))
        .service(route::auth::api_v1_scope())
        .service(route::key::api_v1_scope())
        .service(route::service::api_v1_scope())
        .service(route::user::api_v1_scope())
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
        Ok(_res) => future::ok(HttpResponse::Ok().finish()),
        Err(err) => future::ok(err.error_response()),
    }
}

/// Route response handler.
pub fn route_response_json<T: Serialize>(
    result: Result<T, Error>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    match result {
        Ok(res) => future::ok(HttpResponse::Ok().json(res)),
        Err(err) => future::ok(err.error_response()),
    }
}
