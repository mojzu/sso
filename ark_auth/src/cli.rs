use crate::crate_user_agent;
use crate::driver::Driver;
use crate::notify::NotifyExecutor;
use crate::{core, notify, server};
use actix_rt::System;

/// Configuration.
pub struct Configuration {
    pub notify: notify::Configuration,
    pub server: server::Configuration,
}

impl Configuration {
    /// Create new configuration.
    pub fn new(bind: String) -> Self {
        Configuration {
            notify: notify::Configuration::default(),
            server: server::Configuration::new(bind),
        }
    }

    /// Get reference to notify configuration.
    pub fn notify(&self) -> &notify::Configuration {
        &self.notify
    }

    /// Get mutable reference to notify configuration.
    pub fn mut_notify(&mut self) -> &mut notify::Configuration {
        &mut self.notify
    }

    /// Get reference to server configuration.
    pub fn server(&self) -> &server::Configuration {
        &self.server
    }

    /// Get mutable reference to server configuration.
    pub fn mut_server(&mut self) -> &mut server::Configuration {
        &mut self.server
    }
}

/// Create a root key.
pub fn create_root_key(driver: Box<Driver>, name: &str) -> Result<core::Key, core::Error> {
    let mut audit = audit_builder();
    core::key::create_root(driver.as_ref(), &mut audit, true, name)
}

/// Delete all root keys.
pub fn delete_root_keys(driver: Box<Driver>) -> Result<usize, core::Error> {
    let mut audit = audit_builder();
    core::key::delete_root(driver.as_ref(), &mut audit)
}

/// Create a service with service key.
pub fn create_service_with_key(
    driver: Box<Driver>,
    name: &str,
    url: &str,
) -> Result<(core::Service, core::Key), core::Error> {
    let mut audit = audit_builder();
    let service = core::service::create(driver.as_ref(), &mut audit, true, name, url)?;
    let key = core::key::create_service(driver.as_ref(), &mut audit, true, name, &service.id)?;
    Ok((service, key))
}

/// Start server.
pub fn start_server(
    driver: Box<Driver>,
    configuration: Configuration,
) -> Result<(), server::Error> {
    let system = System::new(crate_name!());

    // Start notify actor.
    let notify_configuration = configuration.notify().clone();
    let notify_addr = NotifyExecutor::start(2, notify_configuration);

    // Start HTTP server.
    let server_configuration = configuration.server().clone();
    let server_notify_addr = notify_addr.clone();
    server::start(4, server_configuration, driver, server_notify_addr)?;

    system.run().map_err(server::Error::StdIo)
}

pub fn audit_builder() -> core::audit::AuditBuilder {
    core::audit::AuditBuilder::new(core::AuditMeta {
        user_agent: crate_user_agent(),
        remote: "127.0.0.1".to_owned(),
        forwarded_for: None,
    })
}
