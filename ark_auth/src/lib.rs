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

// TODO(doc): Update manual with init -> create,delete changes.

/// Create a root key.
pub fn command_create(driver: Box<driver::Driver>, name: &str) -> Result<core::Key, core::Error> {
    core::key::create_root(driver.as_ref(), name)
}

/// Delete all root keys.
pub fn command_delete(driver: Box<driver::Driver>) -> Result<usize, core::Error> {
    core::key::delete_root(driver.as_ref())
}

/// Start server.
pub fn command_start(
    driver: Box<driver::Driver>,
    configuration: server::Configuration,
) -> Result<(), server::Error> {
    actix_rt::System::run(move || {
        server::start(configuration, driver).unwrap();
    })
    .map_err(server::Error::StdIo)
}
