//! Single Sign-On Library
#![recursion_limit = "1024"]
#![deny(missing_debug_implementations)]
// TODO(docs): Require documentation, better library interface.
// #![deny(missing_docs)]

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

mod driver;
pub mod grpc;

pub use crate::driver::*;

/// Implement `to_string` and `from_string` on simple enums.
///
/// Enums must implement serde `Serialize` and `Deserialize` traits.
/// Prefix can be used or empty string `""` for none.
#[macro_export]
macro_rules! impl_enum_to_from_string {
    ($x:ident, $prefix:expr) => {
        impl $x {
            /// Format as string.
            pub fn to_string(self) -> ::serde_json::Result<String> {
                let s = ::serde_json::to_string(&self)?;
                let trim = s.trim_matches('"');
                Ok(format!("{}{}", $prefix, trim))
            }

            /// Parse from string.
            pub fn from_string<S: Into<String>>(s: S) -> ::serde_json::Result<Self> {
                let mut s: String = s.into();
                let s = format!("\"{}\"", s.split_off($prefix.len()));
                ::serde_json::from_str(&s)
            }
        }
    };
}

// TODO(refactor2): Check manual guides, update as needed.
