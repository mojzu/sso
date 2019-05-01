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

pub mod api;
pub mod db;
pub mod driver;
mod email;
pub mod models;
mod schema;

use crate::api::ApiConfig;
use crate::models::{AuthKey, AuthService};

/// Command line interface errors.
#[derive(Fail, Debug)]
pub enum CliError {
    /// Command is invalid.
    #[fail(display = "CliError::InvalidCommand")]
    InvalidCommand,
    /// Database module error wrapper.
    #[fail(display = "CliError::Db {}", _0)]
    Db(#[fail(cause)] db::DbError),
    /// Standard IO error wrapper.
    #[fail(display = "CliError::StdIo {}", _0)]
    StdIo(#[fail(cause)] std::io::Error),
    /// Standard environment variable error wrapper.
    #[fail(display = "CliError::StdEnvVar {}", _0)]
    StdEnvVar(#[fail(cause)] std::env::VarError),
}

/// Initialise a new service with name and URL, generates a key for created service.
pub fn cli_init(db_url: &str, name: &str, url: &str) -> Result<(AuthService, AuthKey), CliError> {
    let db = db::Db::new(&db_url);
    let service = db.service_create(name, url).map_err(CliError::Db)?;
    let key = db
        .key_create(name, service.service_id, None)
        .map_err(CliError::Db)?;
    Ok((service, key))
}

/// Start API server in system.
pub fn cli_start(config: ApiConfig, db_url: &str) -> Result<(), CliError> {
    let db = db::Db::new(&db_url);
    actix_rt::System::run(move || {
        api::start(config, db);
    })
    .map_err(CliError::StdIo)
}
