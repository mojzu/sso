use chrono::{DateTime, Utc};
use std::fmt;
use time::Duration;
use uuid::Uuid;

/// CSRF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Csrf {
    pub created_at: DateTime<Utc>,
    pub key: String,
    pub value: String,
    pub ttl: DateTime<Utc>,
    pub service_id: Uuid,
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
