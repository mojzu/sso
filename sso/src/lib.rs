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

mod driver;
pub mod grpc;

pub use crate::driver::*;

use sentry::integrations::log::LoggerOptions;
use std::io::Write;

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

/// Initialise logging from named environment variables.
///
/// Sentry integration is enabled if `sentry_dsn_name` environment variable is defined.
/// Logs are formatted as single line JSON objects by defaullt, for integration with
/// fluentd. Logs are formatted as coloured, multi-line JSON objects if `pretty_name`
/// environment variable is set to `true`.
pub fn log_init<S, P>(
    sentry_dsn_name: S,
    pretty_name: P,
) -> Option<sentry::internals::ClientInitGuard>
where
    S: AsRef<str>,
    P: AsRef<str>,
{
    let pretty = env::value_opt::<bool>(pretty_name.as_ref())
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

    match env::string_opt(sentry_dsn_name.as_ref()) {
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
