//! # Server
//! HTTP server.
pub mod user;

use crate::driver;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware,
    middleware::identity::{IdentityPolicy, IdentityService},
    web, App, HttpResponse, HttpServer,
};
use futures::future;
use serde::de::DeserializeOwned;
use validator::{Validate};

/// Server errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Bad request, deserialisation failure.
    #[fail(display = "ServerError::BadRequest")]
    BadRequest,
    /// Forbidden, authentication failure.
    #[fail(display = "ServerError::Forbidden")]
    Forbidden,
    /// Driver error wrapper.
    #[fail(display = "ServerError::DriverError {}", _0)]
    Driver(#[fail(cause)] driver::Error),
    /// Actix web blocking error cancelled wrapper.
    #[fail(display = "ServerError::ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,
    /// Standard IO error wrapper.
    #[fail(display = "ServerError::StdIoError {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
}

impl From<driver::Error> for Error {
    fn from(error: driver::Error) -> Self {
        Error::Driver(error)
    }
}

impl actix_web::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
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

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    bind: String,
    user_agent: String,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(bind: String) -> Self {
        Configuration {
            bind,
            user_agent: Configuration::default_user_agent(),
        }
    }

    /// Configured bind address.
    pub fn bind(&self) -> &str {
        &self.bind
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

/// Route body JSON size limit configuration.
pub fn body_json_config() -> web::JsonConfig {
    web::JsonConfig::default().limit(1024)
}

/// Validate JSON value trait.
trait ValidateJsonValue<T: DeserializeOwned + Validate> {
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

pub mod validate {
    use validator::{ValidationError};

    pub fn unsigned(id: i64) -> Result<(), ValidationError> {
        if id < 0 {
            Err(ValidationError::new("invalid_unsigned"))
        } else {
            Ok(())
        }
    }
}
