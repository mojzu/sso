use crate::{CoreError, CoreResult};
use serde::{de::DeserializeOwned, ser::Serialize};

/// Core utility functions.
#[derive(Debug)]
pub struct CoreUtil;

impl CoreUtil {
    /// Serialise a query string.
    pub fn qs_ser<T: Serialize>(v: &T) -> CoreResult<String> {
        serde_qs::to_string(v).map_err(CoreError::serde_qs)
    }

    /// Deserialise a query string.
    pub fn qs_de<T: DeserializeOwned>(s: &str) -> CoreResult<T> {
        serde_qs::from_str(s).map_err(CoreError::serde_qs)
    }
}

/// Implement `to_string` and `from_string` on simple enums that implement
/// serde `Serialize` and `Deserialize` traits.
#[macro_export]
macro_rules! impl_enum_to_from_string {
    ($x:ident, $prefix:expr) => {
        impl $x {
            pub fn to_string(self) -> CoreResult<String> {
                let s = serde_json::to_string(&self).map_err(CoreError::SerdeJson)?;
                let trim = s.trim_matches('"');
                Ok(format!("{}{}", $prefix, trim))
            }

            pub fn from_string<S: Into<String>>(s: S) -> CoreResult<Self> {
                let mut s: String = s.into();
                let s = format!("\"{}\"", s.split_off($prefix.len()));
                serde_json::from_str(&s).map_err(CoreError::SerdeJson)
            }
        }
    };
}
