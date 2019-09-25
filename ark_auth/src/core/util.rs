use crate::{CoreError, CoreResult};
use serde::{de::DeserializeOwned, ser::Serialize};
use serde_json::Value;

/// Core utility functions.
pub struct CoreUtil;

impl CoreUtil {
    /// Serialise a query string.
    pub fn qs_ser<T: Serialize>(v: &T) -> CoreResult<String> {
        serde_qs::to_string(v).map_err(Into::into)
    }

    /// Deserialise a query string.
    pub fn qs_de<T: DeserializeOwned>(s: &str) -> CoreResult<T> {
        serde_qs::from_str(s).map_err(Into::into)
    }

    /// Deserialise value as a query string.
    pub fn qs_de_value<T: DeserializeOwned>(v: &Value) -> CoreResult<T> {
        let s = serde_json::to_string(v).map_err(CoreError::SerdeJson)?;
        serde_qs::from_str(&s).map_err(Into::into)
    }
}
