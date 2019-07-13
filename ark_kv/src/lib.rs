//! # ark_kv
//! Library for binary application.
#![recursion_limit = "1024"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate failure;
// #[macro_use]
// extern crate log;
// #[macro_use]
// extern crate serde_derive;

pub mod cli;
pub mod core;
pub mod driver;
