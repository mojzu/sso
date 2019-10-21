use crate::{CoreError, DriverError};
use std::env::VarError;

/// Library errors.
#[derive(Debug, Fail)]
pub enum SsoError {
    #[fail(display = "SsoError:Core {}", _0)]
    Core(#[fail(cause)] CoreError),

    #[fail(display = "SsoError:Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

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

impl From<DriverError> for SsoError {
    fn from(e: DriverError) -> Self {
        Self::Driver(e)
    }
}
