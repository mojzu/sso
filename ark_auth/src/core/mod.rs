mod audit;
mod auth;
mod csrf;
mod jwt;
mod key;
mod metrics;
mod service;
mod user;

pub use crate::core::{
    audit::*, auth::*, csrf::*, jwt::*, key::*, metrics::*, service::*, user::*,
};

use crate::{ClientError, DriverError};
use actix::MailboxError as ActixMailboxError;
use libreauth::pass::ErrorCode as LibreauthPassError;
use zxcvbn::ZxcvbnError;

/// Core errors.
#[derive(Debug, Fail)]
pub enum CoreError {
    #[fail(display = "CoreError::BadRequest")]
    BadRequest,

    #[fail(display = "CoreError::Forbidden")]
    Forbidden,

    #[fail(display = "ServerError:PwnedPasswordsDisabled")]
    PwnedPasswordsDisabled,

    #[fail(display = "CoreError::Cast")]
    Cast,

    #[fail(display = "CoreError::Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

    #[fail(display = "CoreError::LibreauthPass {}", _0)]
    LibreauthPass(usize),

    #[fail(display = "CoreError::Jsonwebtoken {}", _0)]
    Jsonwebtoken(#[fail(cause)] jsonwebtoken::errors::Error),

    #[fail(display = "CoreError::UuidParse {}", _0)]
    UuidParse(#[fail(cause)] uuid::parser::ParseError),

    #[fail(display = "ServerError:Client {}", _0)]
    Client(#[fail(cause)] ClientError),

    #[fail(display = "ServerError:ActixMailbox {}", _0)]
    ActixMailbox(#[fail(cause)] ActixMailboxError),

    #[fail(display = "ServerError:Zxcvbn {}", _0)]
    Zxcvbn(#[fail(cause)] ZxcvbnError),
}

/// Core result wrapper type.
pub type CoreResult<T> = Result<T, CoreError>;

impl CoreError {
    pub fn libreauth(e: LibreauthPassError) -> Self {
        Self::LibreauthPass(e as usize)
    }
}

/// Core.
pub struct Core;

impl Core {
    /// Default list limit.
    pub fn default_limit() -> i64 {
        50
    }
}
