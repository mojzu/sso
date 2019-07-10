use crate::crate_user_agent;
use crate::driver::Driver;
use crate::notify::NotifyExecutor;
use crate::{core, server};
use actix_rt::System;

// TODO(refactor): Move configuration here, split into notify/server/etc.

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
    configuration: server::Configuration,
) -> Result<(), server::Error> {
    let system = System::new(crate_name!());

    // Start notify actor.
    let notify_addr = NotifyExecutor::start(2);

    // Start HTTP server.
    server::start(4, &configuration, &driver, &notify_addr)?;

    system.run().map_err(server::Error::StdIo)
}

pub fn audit_builder() -> core::audit::AuditBuilder {
    core::audit::AuditBuilder::new(core::AuditMeta {
        user_agent: crate_user_agent(),
        remote: "127.0.0.1".to_owned(),
        forwarded_for: None,
    })
}
