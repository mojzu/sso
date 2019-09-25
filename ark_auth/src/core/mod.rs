mod audit;
mod auth;
mod csrf;
mod jwt;
mod key;
mod metrics;
mod service;
mod user;
mod util;

pub use crate::core::{
    audit::*, auth::*, csrf::*, jwt::*, key::*, metrics::*, service::*, user::*, util::*,
};

use crate::{ClientError, DriverError};
use actix::MailboxError as ActixMailboxError;
use libreauth::{oath::ErrorCode as LibreauthOathError, pass::ErrorCode as LibreauthPassError};
use zxcvbn::ZxcvbnError;

/// Core errors.
#[derive(Debug, Fail)]
pub enum CoreError {
    #[fail(display = "CoreError:BadRequest")]
    BadRequest,

    #[fail(display = "CoreError:Forbidden")]
    Forbidden,

    #[fail(display = "CoreError:PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,

    #[fail(display = "CoreError:Metrics")]
    Metrics,

    #[fail(display = "CoreError:AuditType")]
    AuditType,

    #[fail(display = "CoreError:Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

    #[fail(display = "CoreError:Client {}", _0)]
    Client(#[fail(cause)] ClientError),

    #[fail(display = "CoreError:LibreauthPass {}", _0)]
    LibreauthPass(usize),

    #[fail(display = "CoreError:LibreauthOath {}", _0)]
    LibreauthOath(usize),

    #[fail(display = "CoreError:Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),

    #[fail(display = "CoreError:UuidParse {}", _0)]
    UuidParse(#[fail(cause)] uuid::parser::ParseError),

    #[fail(display = "CoreError:ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] ActixMailboxError),

    #[fail(display = "CoreError:SerdeJson {}", _0)]
    SerdeJson(#[fail(cause)] serde_json::Error),

    #[fail(display = "CoreError:SerdeQs {}", _0)]
    SerdeQs(String),

    #[fail(display = "CoreError:Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] ZxcvbnError),
}

impl CoreError {
    pub fn libreauth_pass(e: LibreauthPassError) -> Self {
        Self::LibreauthPass(e as usize)
    }

    pub fn libreauth_oath(e: LibreauthOathError) -> Self {
        Self::LibreauthOath(e as usize)
    }
}

impl From<DriverError> for CoreError {
    fn from(e: DriverError) -> Self {
        Self::Driver(e)
    }
}

impl From<ClientError> for CoreError {
    fn from(e: ClientError) -> Self {
        Self::Client(e)
    }
}

impl From<serde_qs::Error> for CoreError {
    fn from(e: serde_qs::Error) -> Self {
        Self::SerdeQs(e.description().to_owned())
    }
}

/// Core result wrapper type.
pub type CoreResult<T> = Result<T, CoreError>;

/// Core.
pub struct Core;

impl Core {
    /// Default list limit.
    pub fn default_limit() -> i64 {
        50
    }
}
