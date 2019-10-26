use crate::DriverError;
use std::env::VarError;

// TODO(refactor): Remove this error in favour of DriverError.

/// Library errors.
#[derive(Debug, Fail)]
pub enum SsoError {
    #[fail(display = "SsoError:Driver {}", _0)]
    Driver(#[fail(cause)] DriverError),

    #[fail(display = "SsoError:EnvParse {}", _0)]
    EnvParse(String),

    #[fail(display = "SsoError:StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] VarError),
}

/// Library result wrapper type.
pub type SsoResult<T> = Result<T, SsoError>;

impl From<DriverError> for SsoError {
    fn from(e: DriverError) -> Self {
        Self::Driver(e)
    }
}
