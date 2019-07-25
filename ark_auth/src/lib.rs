//! # ark_auth
//! Library for binary application.
#![recursion_limit = "1024"]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;
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

pub mod cli;
pub mod client;
pub mod core;
pub mod driver;
pub mod notify;
pub mod server;

/// Crate name macro as function.
pub fn crate_name() -> String {
    crate_name!().to_string()
}

/// User agent constructed from crate name and version.
pub fn crate_user_agent() -> String {
    format!("{}/{}", crate_name!(), crate_version!())
}
