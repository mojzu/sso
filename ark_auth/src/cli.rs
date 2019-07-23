use crate::crate_user_agent;
use crate::driver::Driver;
use crate::notify::NotifyExecutor;
use crate::{core, notify, server};
use actix_rt::System;

/// Command line interface errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// Core error wrapper.
    #[fail(display = "CliError::Core {}", _0)]
    Core(#[fail(cause)] core::Error),
    /// Server error wrapper.
    #[fail(display = "CliError::Server {}", _0)]
    Server(#[fail(cause)] server::Error),
    /// Standard environment variable error wrapper.
    #[fail(display = "CliError::StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),
    /// Standard number parse integer error wrapper.
    #[fail(display = "CliError::StdNumParseInt {}", _0)]
    StdNumParseInt(#[fail(cause)] std::num::ParseIntError),
}

/// Configuration.
pub struct Configuration {
    pub notify: notify::Configuration,
    pub server: server::Configuration,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(notify: notify::Configuration, server: server::Configuration) -> Self {
        Self { notify, server }
    }

    /// Get reference to notify configuration.
    pub fn notify(&self) -> &notify::Configuration {
        &self.notify
    }

    /// Get reference to server configuration.
    pub fn server(&self) -> &server::Configuration {
        &self.server
    }
}

/// Create a root key.
pub fn create_root_key(driver: Box<Driver>, name: &str) -> Result<core::Key, Error> {
    let mut audit = audit_builder();
    core::key::create_root(driver.as_ref(), &mut audit, true, name).map_err(Error::Core)
}

/// Delete all root keys.
pub fn delete_root_keys(driver: Box<Driver>) -> Result<usize, Error> {
    let mut audit = audit_builder();
    core::key::delete_root(driver.as_ref(), &mut audit).map_err(Error::Core)
}

/// Create a service with service key.
pub fn create_service_with_key(
    driver: Box<Driver>,
    name: &str,
    url: &str,
) -> Result<(core::Service, core::Key), Error> {
    let mut audit = audit_builder();
    let service =
        core::service::create(driver.as_ref(), &mut audit, true, name, url).map_err(Error::Core)?;
    let key = core::key::create_service(driver.as_ref(), &mut audit, true, name, &service.id)
        .map_err(Error::Core)?;
    Ok((service, key))
}

/// Start server.
pub fn start_server(driver: Box<Driver>, configuration: Configuration) -> Result<(), Error> {
    let system = System::new(crate_name!());

    // Start notify actor.
    let notify_configuration = configuration.notify().clone();
    let notify_addr = NotifyExecutor::start(2, notify_configuration);

    // Start HTTP server.
    let server_configuration = configuration.server().clone();
    let server_notify_addr = notify_addr.clone();
    server::start(4, server_configuration, driver, server_notify_addr).map_err(Error::Server)?;

    system
        .run()
        .map_err(server::Error::StdIo)
        .map_err(Error::Server)
}

pub fn audit_builder() -> core::audit::AuditBuilder {
    core::audit::AuditBuilder::new(core::AuditMeta::new(
        crate_user_agent(),
        "127.0.0.1".to_owned(),
        None,
    ))
}
