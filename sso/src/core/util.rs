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
    ($x:ident) => {
        impl $x {
            pub fn to_string(self) -> CoreResult<String> {
                let s = serde_json::to_string(&self).map_err(CoreError::SerdeJson)?;
                Ok(String::from(s.trim_matches('"')))
            }

            pub fn from_string(s: &str) -> CoreResult<Self> {
                let s = format!("\"{}\"", s);
                serde_json::from_str(&s).map_err(CoreError::SerdeJson)
            }
        }
    };
}
