use actix::MailboxError as ActixMailboxError;
use http::StatusCode;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use serde_urlencoded::ser::Error as SerdeUrlencodedSerError;
use std::error::Error as StdError;
use std::io::Error as StdIoError;

/// ## Client Errors
#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    /// Bad request error.
    #[fail(display = "ClientError:BadRequest")]
    BadRequest,
    /// Forbidden error.
    #[fail(display = "ClientError:Forbidden")]
    Forbidden,
    /// Not found error.
    #[fail(display = "ClientError:NotFound")]
    NotFound,
    /// Client error.
    #[fail(display = "ClientError:Client {}", _0)]
    Client(String),
    /// Serde JSON error wrapper.
    #[fail(display = "ClientError:SerdeJson {}", _0)]
    SerdeJson(String),
    /// Serde URL encoded serialise error wrapper.
    #[fail(display = "ClientError:SerdeUrlencodedSer {}", _0)]
    SerdeUrlencodedSer(#[fail(cause)] SerdeUrlencodedSerError),
    /// Url error wrapper.
    #[fail(display = "ClientError:Url {}", _0)]
    Url(String),
    /// Actix mailbox error wrapper.
    #[fail(display = "ClientError:ActixMailbox {}", _0)]
    ActixMailbox(String),
    /// Standard IO error wrapper.
    #[fail(display = "ClientError:StdIo {}", _0)]
    StdIo(String),
}

impl Error {
    pub fn url(err: &StdError) -> Error {
        Error::Url(err.description().into())
    }

    pub fn stdio(err: &StdIoError) -> Error {
        Error::StdIo(err.description().into())
    }
}

impl From<ReqwestError> for Error {
    fn from(e: ReqwestError) -> Error {
        if let Some(status) = e.status() {
            match status {
                StatusCode::BAD_REQUEST => Error::BadRequest,
                StatusCode::FORBIDDEN => Error::Forbidden,
                StatusCode::NOT_FOUND => Error::NotFound,
                _ => Error::Client(e.description().to_owned()),
            }
        } else {
            Error::Client(e.description().to_owned())
        }
    }
}

impl From<SerdeJsonError> for Error {
    fn from(e: SerdeJsonError) -> Error {
        Error::SerdeJson(e.description().to_owned())
    }
}

impl From<ActixMailboxError> for Error {
    fn from(e: ActixMailboxError) -> Error {
        Error::ActixMailbox(e.description().to_owned())
    }
}
