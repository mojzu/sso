//! Library for binary application.
#![recursion_limit = "1024"]
// TODO(docs): Require documentation.
// #![deny(missing_docs)]

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
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate validator_derive;

mod cli;
mod client;
mod core;
mod driver;
mod env;
mod notify;
mod result;
mod server;

pub use crate::{cli::*, client::*, core::*, driver::*, env::*, notify::*, result::*, server::*};
