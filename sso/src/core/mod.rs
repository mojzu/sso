mod audit;
mod auth;
mod csrf;
mod error;
mod jwt;
mod key;
mod metrics;
mod service;
mod user;
mod util;

pub use crate::core::{
    audit::*, auth::*, csrf::*, error::*, jwt::*, key::*, metrics::*, service::*, user::*, util::*,
};

/// Core functions.
#[derive(Debug)]
pub struct Core;

impl Core {
    /// Default list limit.
    pub fn default_limit() -> i64 {
        50
    }

    /// Default CSRF expires in seconds.
    pub fn default_csrf_expires_s() -> i64 {
        1000
    }
}
