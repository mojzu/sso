pub mod route;
pub mod smtp;
pub mod validate;

use crate::crate_user_agent;
use crate::{core, driver};
use actix_identity::{IdentityPolicy, IdentityService};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{middleware, web, App, HttpResponse, HttpServer, ResponseError};
use serde::Serialize;
pub use validate::FromJsonValue;

// TODO(feature): Audit logging, x-real-ip, x-forwarded-for headers.
// TODO(feature): Prometheus metrics.
// <https://prometheus.io/docs/instrumenting/exposition_formats/>
// TODO(feature): Webauthn support.
// <https://webauthn.guide/>

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

/// SMTP errors.
#[derive(Debug, Fail)]
pub enum SmtpError {
    /// Integration disabled.
    #[fail(display = "SmtpError::Disabled")]
    Disabled,
    /// Native TLS error.
    #[fail(display = "SmtpError::NativeTls {}", _0)]
    NativeTls(native_tls::Error),
    /// Lettre email error.
    #[fail(display = "SmtpError::LettreEmail {}", _0)]
    LettreEmail(lettre_email::error::Error),
    /// Lettre error.
    #[fail(display = "SmtpError::Lettre {}", _0)]
    Lettre(lettre::smtp::error::Error),
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
    /// SMTP error.
    #[fail(display = "ServerError::Smtp {}", _0)]
    Smtp(SmtpError),
    /// OAuth2 error.
    #[fail(display = "ServerError::Oauth2 {}", _0)]
    Oauth2(Oauth2Error),
    /// Core error wrapper.
    #[fail(display = "ServerError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
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

/// Provider OAuth2 configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProviderOauth2 {
    client_id: String,
    client_secret: String,
    redirect_url: String,
}

/// Provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProvider {
    oauth2: Option<ConfigurationProviderOauth2>,
}

// Provider group configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationProviderGroup {
    github: ConfigurationProvider,
    microsoft: ConfigurationProvider,
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
    token_expiration_time: usize,
    password_pwned_enabled: bool,
    smtp: Option<ConfigurationSmtp>,
    provider: ConfigurationProviderGroup,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(bind: String) -> Self {
        Configuration {
            bind,
            user_agent: crate_user_agent(),
            token_expiration_time: 3600,
            password_pwned_enabled: false,
            smtp: None,
            provider: ConfigurationProviderGroup {
                github: ConfigurationProvider { oauth2: None },
                microsoft: ConfigurationProvider { oauth2: None },
            },
        }
    }

    /// Set token expiry time in seconds (defaults to 1 hour).
    pub fn set_token_expiration_time(mut self, value: usize) -> Self {
        self.token_expiration_time = value;
        self
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

    /// Set provider GitHub OAuth2.
    pub fn set_provider_github_oauth2(
        mut self,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        self.provider.github.oauth2 = Some(ConfigurationProviderOauth2 {
            client_id,
            client_secret,
            redirect_url,
        });
        self
    }

    /// Set provider Microsoft OAuth2.
    pub fn set_provider_microsoft_oauth2(
        mut self,
        client_id: String,
        client_secret: String,
        redirect_url: String,
    ) -> Self {
        self.provider.microsoft.oauth2 = Some(ConfigurationProviderOauth2 {
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

    /// Get token expiry.
    pub fn token_expiration_time(&self) -> usize {
        self.token_expiration_time
    }

    /// Get password pwned enabled.
    pub fn password_pwned_enabled(&self) -> bool {
        self.password_pwned_enabled
    }

    /// Configured SMTP provider.
    pub fn smtp(&self) -> Option<&ConfigurationSmtp> {
        self.smtp.as_ref()
    }

    /// Configured provider GitHub OAuth2.
    pub fn provider_github_oauth2(&self) -> Option<&ConfigurationProviderOauth2> {
        self.provider.github.oauth2.as_ref()
    }

    /// Configured provider Microsoft OAuth2.
    pub fn provider_microsoft_oauth2(&self) -> Option<&ConfigurationProviderOauth2> {
        self.provider.microsoft.oauth2.as_ref()
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
            // Route service.
            .configure(route::route_service)
            // Default route (method not allowed).
            .default_service(web::route().to(HttpResponse::MethodNotAllowed))
    })
    .bind(bind)
    .map_err(Error::StdIo)?;

    server.start();
    Ok(())
}
