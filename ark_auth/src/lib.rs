//! # Ark Auth
//! Library for binary application.
#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate validator_derive;

pub mod core;
pub mod driver;
pub mod server;

/// Initialise a new service with name and URL, generates a key for created service.
pub fn command_init(
    driver: Box<driver::Driver>,
    name: &str,
    url: &str,
) -> Result<(core::Service, core::Key), core::Error> {
    let service = core::service::create(driver.as_ref(), name, url)?;
    let key = core::key::create(driver.as_ref(), &service, name, None)?;
    Ok((service, key))
}

/// Start API server.
pub fn command_start(
    configuration: server::Configuration,
    driver: Box<driver::Driver>,
) -> Result<(), server::Error> {
    actix_rt::System::run(move || {
        server::start(configuration, driver).unwrap();
    })
    .map_err(server::Error::StdIo)
}
