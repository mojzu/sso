use crate::CoreError;
use http::StatusCode;
use std::{error::Error, io::Error as IoError};

/// Client errors.
#[derive(Debug, Fail, PartialEq)]
pub enum ClientError {
    #[fail(display = "ClientError:BadRequest")]
    BadRequest,

    #[fail(display = "ClientError:Unauthorised")]
    Unauthorised,

    #[fail(display = "ClientError:Forbidden")]
    Forbidden,

    #[fail(display = "ClientError:NotFound")]
    NotFound,

    #[fail(display = "ClientError:Client {}", _0)]
    Client(String),

    #[fail(display = "ClientError:Core {}", _0)]
    Core(String),

    #[fail(display = "ClientError:SerdeJson {}", _0)]
    SerdeJson(String),

    #[fail(display = "ClientError:Url {}", _0)]
    Url(String),

    #[fail(display = "ClientError:ActixMailbox {}", _0)]
    ActixMailbox(String),

    #[fail(display = "ClientError:StdIo {}", _0)]
    StdIo(String),
}

/// Client result wrapper type.
pub type ClientResult<T> = Result<T, ClientError>;

impl ClientError {
    pub fn core(e: CoreError) -> Self {
        Self::Core(format!("{}", e))
    }

    pub fn url(e: &dyn Error) -> Self {
        Self::Url(e.description().into())
    }

    pub fn stdio(e: &IoError) -> Self {
        Self::StdIo(e.description().into())
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        if let Some(status) = e.status() {
            match status {
                StatusCode::BAD_REQUEST => Self::BadRequest,
                StatusCode::UNAUTHORIZED => Self::Unauthorised,
                StatusCode::FORBIDDEN => Self::Forbidden,
                StatusCode::NOT_FOUND => Self::NotFound,
                _ => Self::Client(e.description().to_owned()),
            }
        } else {
            Self::Client(e.description().to_owned())
        }
    }
}

impl From<serde_json::Error> for ClientError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeJson(e.description().to_owned())
    }
}

impl From<actix::MailboxError> for ClientError {
    fn from(e: actix::MailboxError) -> Self {
        Self::ActixMailbox(e.description().to_owned())
    }
}
