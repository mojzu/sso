use crate::client::ClientError;
use crate::core::Error as CoreError;
use actix::MailboxError as ActixMailboxError;
use actix_web::error::BlockingError as ActixWebBlockingError;
use actix_web::{HttpResponse, ResponseError};
use failure::Error as FailureError;
use prometheus::Error as PrometheusError;
use std::io::Error as StdIoError;
use url::ParseError as UrlParseError;
use zxcvbn::ZxcvbnError;

/// OAuth2 errors.
#[derive(Debug, Fail)]
pub enum Oauth2Error {
    /// Integration disabled.
    #[fail(display = "Oauth2Error:Disabled")]
    Disabled,
    /// CSRF error.
    #[fail(display = "Oauth2Error:Csrf")]
    Csrf,
    /// OAuth2 request token error.
    #[fail(display = "Oauth2Error:Oauth2Request {}", _0)]
    Oauth2Request(FailureError),
}

/// Server errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Bad request, deserialisation failure.
    #[fail(display = "ServerError:BadRequest")]
    BadRequest,
    /// Forbidden, authentication failure.
    #[fail(display = "ServerError:Forbidden")]
    Forbidden,
    /// Not found.
    #[fail(display = "ServerError:NotFound")]
    NotFound,
    /// Pwned passwords disabled error.
    #[fail(display = "ServerError:PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,
    /// OAuth2 error.
    #[fail(display = "ServerError:Oauth2 {}", _0)]
    Oauth2(Oauth2Error),
    /// Core error wrapper.
    #[fail(display = "ServerError:Core {}", _0)]
    Core(#[fail(cause)] CoreError),
    /// Client error wrapper.
    #[fail(display = "ServerError:Client {}", _0)]
    Client(#[fail(cause)] ClientError),
    /// Rustls error wrapper.
    #[fail(display = "ServerError:Rustls")]
    Rustls,
    /// URL parse error.
    #[fail(display = "ServerError:UrlParse {}", _0)]
    UrlParse(#[fail(cause)] UrlParseError),
    /// Actix web blocking error cancelled wrapper.
    #[fail(display = "ServerError:ActixWebBlockingCancelled")]
    ActixWebBlockingCancelled,
    /// Actix mailbox error wrapper.
    #[fail(display = "ServerError:ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] ActixMailboxError),
    /// Standard IO error wrapper.
    #[fail(display = "ServerError:StdIo {}", _0)]
    StdIo(#[fail(cause)] StdIoError),
    /// Zxcvbn error wrapper.
    #[fail(display = "ServerError:Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] ZxcvbnError),
    /// Prometheus error wrapper.
    #[fail(display = "ServerError:Prometheus {}", _0)]
    Prometheus(#[fail(cause)] PrometheusError),
}

impl From<CoreError> for Error {
    fn from(e: CoreError) -> Self {
        match e {
            CoreError::BadRequest => Error::BadRequest,
            CoreError::Forbidden => Error::Forbidden,
            CoreError::Jsonwebtoken(_e) => Error::BadRequest,
            _ => Error::Core(e),
        }
    }
}

impl From<ActixWebBlockingError<Error>> for Error {
    fn from(e: ActixWebBlockingError<Error>) -> Self {
        match e {
            ActixWebBlockingError::Error(e) => e,
            ActixWebBlockingError::Canceled => Error::ActixWebBlockingCancelled,
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
