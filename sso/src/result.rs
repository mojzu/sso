use crate::{CoreError, ServerError};
use std::env::VarError;

/// Library errors.
#[derive(Debug, Fail)]
pub enum SsoError {
    #[fail(display = "SsoError:Core {}", _0)]
    Core(#[fail(cause)] CoreError),

    #[fail(display = "SsoError:Server {}", _0)]
    Server(#[fail(cause)] ServerError),

    #[fail(display = "SsoError:EnvParse {}", _0)]
    EnvParse(String),

    #[fail(display = "SsoError:StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] VarError),
}

/// Library result wrapper type.
pub type SsoResult<T> = Result<T, SsoError>;

impl From<CoreError> for SsoError {
    fn from(e: CoreError) -> Self {
        Self::Core(e)
    }
}

impl From<ServerError> for SsoError {
    fn from(e: ServerError) -> Self {
        Self::Server(e)
    }
}
