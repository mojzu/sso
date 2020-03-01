//! # Single Sign-On (Library)
#![recursion_limit = "1024"]
#![deny(missing_debug_implementations)]

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

mod csrf;
mod driver;
mod env;
mod grpc;
pub mod header;
mod http_server;
mod jwt;
mod prelude;
mod schema;
pub mod validate;

pub use crate::driver::*;
pub use crate::{csrf::*, env::*, grpc::*, http_server::*, jwt::*};

use sentry::integrations::log::LoggerOptions;
use std::io::Write;

/// Implement `to_string` and `from_str` on simple enums.
///
/// Enums must implement serde `Serialize` and `Deserialize` traits.
/// Prefix can be used or empty string `""` for none.
#[macro_export]
macro_rules! impl_enum_to_from_string {
    ($x:ident, $prefix:expr) => {
        impl ::std::fmt::Display for $x {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let s = ::serde_json::to_string(&self).map_err(|_| ::std::fmt::Error)?;
                let trim = s.trim_matches('"');
                write!(f, "{}{}", $prefix, trim)
            }
        }
        impl ::std::str::FromStr for $x {
            type Err = ::serde_json::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let mut s: String = s.into();
                let s = format!("\"{}\"", s.split_off($prefix.len()));
                ::serde_json::from_str(&s)
            }
        }
    };
}

/// Initialise logging from named environment variables.
///
/// Sentry integration is enabled if `sentry_dsn_name` environment variable is defined.
/// Logs are formatted as single line JSON objects by defaullt, for integration with
/// fluentd. Logs are formatted as coloured, multi-line JSON objects if `pretty_name`
/// environment variable is set to `true`.
pub fn log_init<T>(sentry_dsn_name: T, pretty_name: T) -> Option<sentry::internals::ClientInitGuard>
where
    T: AsRef<str>,
{
    let pretty = Env::value_opt::<bool>(pretty_name.as_ref())
        .expect("Failed to read environment variable.")
        .unwrap_or(false);

    let mut builder = env_logger::Builder::from_default_env();
    builder.format(move |buf, record| {
        let out = json!({
            "time": chrono::Utc::now().to_rfc3339(),
            "level": record.level().to_string(),
            "target": record.target(),
            "module_path": record.module_path(),
            "file": record.file(),
            "line": record.line(),
            "message": record.args(),
        });
        let out = if pretty {
            serde_json::to_string_pretty(&out)
        } else {
            serde_json::to_string(&out)
        }
        .expect("Failed to serialise log.");

        if pretty {
            let style = buf.default_level_style(record.level());
            writeln!(buf, "{}", style.value(out))
        } else {
            writeln!(buf, "{}", out)
        }
    });

    match Env::string_opt(sentry_dsn_name.as_ref()) {
        Some(sentry_dsn) => {
            let guard = sentry::init(sentry_dsn);
            let mut options = LoggerOptions::default();
            options.emit_warning_events = true;

            sentry::integrations::env_logger::init(Some(builder.build()), options);
            sentry::integrations::panic::register_panic_handler();
            Some(guard)
        }
        None => {
            builder.init();
            warn!("Sentry DSN is undefined, integration is disabled.");
            None
        }
    }
}
