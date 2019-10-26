//! Library for binary application.
#![recursion_limit = "1024"]
#![deny(missing_debug_implementations)]
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

pub mod api;
mod cli;
mod client;
mod driver;
mod env;
mod notify;
mod result;
mod server;

pub use crate::{cli::*, client::*, driver::*, env::*, notify::*, result::*, server::*};

/// Implement `to_string` and `from_string` on simple enums.
///
/// Enums must implement serde `Serialize` and `Deserialize` traits.
/// Prefix can be used or provided empty reference `""` for none.
#[macro_export]
macro_rules! impl_enum_to_from_string {
    ($x:ident, $prefix:expr) => {
        use $crate::{DriverError, DriverResult};
        impl $x {
            pub fn to_string(self) -> DriverResult<String> {
                let s = serde_json::to_string(&self).map_err(DriverError::SerdeJson)?;
                let trim = s.trim_matches('"');
                Ok(format!("{}{}", $prefix, trim))
            }

            pub fn from_string<S: Into<String>>(s: S) -> DriverResult<Self> {
                let mut s: String = s.into();
                let s = format!("\"{}\"", s.split_off($prefix.len()));
                serde_json::from_str(&s).map_err(DriverError::SerdeJson)
            }
        }
    };
}
