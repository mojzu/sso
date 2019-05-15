use crate::{core, driver, server};

/// Create a root key.
pub fn create_root_key(driver: Box<driver::Driver>, name: &str) -> Result<core::Key, core::Error> {
    core::key::create_root(driver.as_ref(), name)
}

/// Delete all root keys.
pub fn delete_root_keys(driver: Box<driver::Driver>) -> Result<usize, core::Error> {
    core::key::delete_root(driver.as_ref())
}

/// Create a service with service key.
pub fn create_service_with_key(
    driver: Box<driver::Driver>,
    name: &str,
    url: &str,
) -> Result<(core::Service, core::Key), core::Error> {
    let service = core::service::create(driver.as_ref(), name, url)?;
    let key = core::key::create_service(driver.as_ref(), name, service.id)?;
    Ok((service, key))
}

/// Start server.
pub fn start_server(
    driver: Box<driver::Driver>,
    configuration: server::Configuration,
) -> Result<(), server::Error> {
    actix_rt::System::run(move || {
        server::start(configuration, driver).unwrap();
    })
    .map_err(server::Error::StdIo)
}
