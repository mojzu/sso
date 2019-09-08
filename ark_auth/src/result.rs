use crate::{CoreError, ServerError};
use std::env::VarError;

/// Library errors.
#[derive(Debug, Fail)]
pub enum AuthError {
    #[fail(display = "AuthError:Core {}", _0)]
    Core(#[fail(cause)] CoreError),

    #[fail(display = "AuthError:Server {}", _0)]
    Server(#[fail(cause)] ServerError),

    #[fail(display = "AuthError:EnvParse {}", _0)]
    EnvParse(String),

    #[fail(display = "AuthError:StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] VarError),
}

/// Library result wrapper type.
pub type AuthResult<T> = Result<T, AuthError>;

impl From<CoreError> for AuthError {
    fn from(e: CoreError) -> Self {
        Self::Core(e)
    }
}

impl From<ServerError> for AuthError {
    fn from(e: ServerError) -> Self {
        Self::Server(e)
    }
}
