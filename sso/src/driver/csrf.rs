use crate::{DriverError, DriverResult};
use chrono::{DateTime, Duration, Utc};
use libreauth::key::KeyBuilder;
use std::fmt;
use uuid::Uuid;

/// CSRF key size in bytes.
pub const BYTES_CSRF_KEY: usize = 11;

/// CSRF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Csrf {
    pub created_at: DateTime<Utc>,
    pub key: String,
    pub value: String,
    pub ttl: DateTime<Utc>,
    pub service_id: Uuid,
}

impl Csrf {
    /// Returns error if service ID does not match CSRF token.
    pub fn check_service(&self, service_id: Uuid) -> DriverResult<()> {
        if service_id != self.service_id {
            Err(DriverError::CsrfServiceMismatch)
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for Csrf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Csrf {}", self.key)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tvalue {}", self.value)?;
        write!(f, "\n\tttl {}", self.ttl)?;
        write!(f, "\n\tservice_id {}", self.service_id)
    }
}

/// CSRF create.
#[derive(Debug)]
pub struct CsrfCreate {
    pub key: String,
    pub value: String,
    pub ttl: DateTime<Utc>,
    pub service_id: Uuid,
}

impl CsrfCreate {
    /// Generate CSRF token with time to live in seconds. Key must be unique.
    pub fn generate(ttl_s: i64, service_id: Uuid) -> Self {
        let key = key_generate();
        Self::new(&key, &key, ttl_s, service_id)
    }

    /// Copy from CSRF token.
    pub fn copy(csrf: Csrf) -> Self {
        Self {
            key: csrf.key,
            value: csrf.value,
            ttl: csrf.ttl,
            service_id: csrf.service_id,
        }
    }

    /// Create CSRF token with time to live in seconds. Key must be unique.
    pub fn new<K, V>(key: K, value: V, ttl_s: i64, service_id: Uuid) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            key: key.into(),
            value: value.into(),
            ttl: Utc::now() + Duration::seconds(ttl_s),
            service_id,
        }
    }
}

/// Generate new key from random bytes.
fn key_generate() -> String {
    KeyBuilder::new()
        .size(BYTES_CSRF_KEY)
        .generate()
        .as_base32()
}
