mod api;
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
    api::*, audit::*, auth::*, csrf::*, error::*, jwt::*, key::*, metrics::*, service::*, user::*,
    util::*,
};

/// Core functions.
pub struct Core;

impl Core {
    /// Default list limit.
    pub fn default_limit() -> i64 {
        50
    }
}
