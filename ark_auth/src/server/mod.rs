//! # Server
//! HTTP server.

use crate::driver;
use actix_web::{App, HttpServer};

/// Server errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Driver error wrapper.
    #[fail(display = "ServerError::DriverError {}", _0)]
    Driver(#[fail(cause)] driver::Error),
    /// Standard IO error wrapper.
    #[fail(display = "ServerError::StdIoError {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
}

impl From<driver::Error> for Error {
    fn from(error: driver::Error) -> Self {
        Error::Driver(error)
    }
}

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    bind: String,
    user_agent: String,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(bind: String) -> Self {
        Configuration {
            bind,
            user_agent: Configuration::default_user_agent(),
        }
    }

    /// Configured bind address.
    pub fn bind(&self) -> &str {
        &self.bind
    }

    /// Default user agent constructed from crate name and version.
    fn default_user_agent() -> String {
        format!("{}/{}", crate_name!(), crate_version!())
    }
}

/// Start HTTP server.
pub fn start(configuration: Configuration) -> Result<(), Error> {
    let bind = configuration.bind().to_owned();

    let server = HttpServer::new(move || App::new())
        .bind(bind)
        .map_err(Error::StdIo)?;

    server.start();
    Ok(())
}
