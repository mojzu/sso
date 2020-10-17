pub(crate) use crate::{
    common::*,
    config::*,
    error::*,
    http_server::*,
    mailto, metrics,
    oauth2::{self, AuthorizationServerIf, ClientIf, SerializeJson, UserRedirectUri},
    postgres::*,
    util, validate,
};
pub(crate) use chrono::{DateTime, Utc};
pub(crate) use opentelemetry::api::metrics::{BoundCounter, BoundValueRecorder};
pub(crate) use paperclip::actix::OpenApiExt;
pub(crate) use serde_json::Value;
pub(crate) use std::{
    borrow::Borrow,
    collections::HashMap,
    convert::{TryFrom, TryInto},
    ops::Deref,
    str::FromStr,
    sync::{Arc, Mutex},
};
pub(crate) use url::Url;
pub(crate) use uuid::Uuid;
pub(crate) use validator::{Validate, ValidationError};

pub(crate) fn default_as_true() -> bool {
    true
}

pub(crate) fn default_as_3600() -> i64 {
    3600
}

pub(crate) fn default_as_86400() -> i64 {
    3600
}

pub(crate) fn default_as_sso() -> String {
    "sso".to_string()
}
