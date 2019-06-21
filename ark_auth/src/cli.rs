use crate::driver::Driver;
use crate::{core, server};
use actix_rt::System;

/// Create a root key.
pub fn create_root_key(driver: Box<Driver>, name: &str) -> Result<core::Key, core::Error> {
    core::key::create_root(driver.as_ref(), true, name)
}

/// Delete all root keys.
pub fn delete_root_keys(driver: Box<Driver>) -> Result<usize, core::Error> {
    core::key::delete_root(driver.as_ref())
}

/// Create a service with service key.
pub fn create_service_with_key(
    driver: Box<Driver>,
    name: &str,
    url: &str,
) -> Result<(core::Service, core::Key), core::Error> {
    let service = core::service::create(driver.as_ref(), true, name, url)?;
    let key = core::key::create_service(driver.as_ref(), true, name, &service.id)?;
    Ok((service, key))
}

/// Start server.
pub fn start_server(
    driver: Box<Driver>,
    configuration: server::Configuration,
) -> Result<(), server::Error> {
    let system = System::new(crate_name!());

    server::start(configuration, driver)?;

    system.run().map_err(server::Error::StdIo)
}
