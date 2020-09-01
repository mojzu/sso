pub use crate::{
    client::*,
    common::*,
    config::*,
    error::*,
    mailto::*,
    oauth2::{self, AuthorizationServerIf, ClientIf, SerializeJson, UserRedirectUri},
    postgres::*,
    server::*,
    util::*,
};
pub use actix_http::error::ResponseError;
pub use chrono::{DateTime, Duration, NaiveDateTime, Utc};
pub use opentelemetry::api::metrics::{BoundCounter, BoundValueRecorder};
pub use paperclip::actix::OpenApiExt;
pub use serde_json::Value;
pub use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    convert::{TryFrom, TryInto},
    ops::Deref,
    sync::{Arc, Mutex},
};
pub use url::Url;
pub use uuid::Uuid;
pub use validator::{Validate, ValidationError};

pub fn default_as_true() -> bool {
    true
}

pub fn default_as_3600() -> i64 {
    3600
}

pub fn default_as_86400() -> i64 {
    3600
}
