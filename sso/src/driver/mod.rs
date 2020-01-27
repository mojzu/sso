mod audit;
mod csrf;
pub mod env;
mod error;
mod jwt;
mod key;
mod metrics;
pub mod pattern;
mod postgres;
mod service;
mod template;
mod user;

pub use crate::driver::postgres::{Postgres, PostgresLockFn};
pub use crate::driver::{
    audit::*, csrf::*, error::*, jwt::*, key::*, metrics::*, service::*, template::*, user::*,
};

/// Default limit.
pub const DEFAULT_LIMIT: i64 = 50;

/// Default CSRF expires seconds.
pub const DEFAULT_CSRF_EXPIRES_S: i64 = 1000;

/// Authorisation header.
pub const HEADER_AUTHORISATION: &str = "Authorization";

/// User authorisation header.
pub const HEADER_USER_AUTHORISATION: &str = "User-Authorization";
