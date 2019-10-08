use crate::{CoreError, CoreResult, Driver, Service};
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

/// CSRF delete.
#[derive(Debug)]
pub enum CsrfDelete {
    Key(String),
    Ttl(DateTime<Utc>),
}

impl Csrf {
    /// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
    pub fn create(
        driver: &dyn Driver,
        service: &Service,
        key: String,
        value: String,
        ttl: i64,
    ) -> CoreResult<Csrf> {
        Csrf::delete_by_ttl(driver)?;

        let ttl = Utc::now() + Duration::seconds(ttl);
        let create = CsrfCreate {
            key,
            value,
            ttl,
            service_id: service.id,
        };
        driver.csrf_create(&create).map_err(CoreError::Driver)
    }

    /// Read one CSRF key, value pair. CSRF key, value pair is deleted after one read.
    pub fn read_opt(driver: &dyn Driver, key: String) -> CoreResult<Option<Csrf>> {
        Csrf::delete_by_ttl(driver)?;

        driver
            .csrf_read_opt(&key)
            .map_err(CoreError::Driver)
            .and_then(|csrf| {
                if csrf.is_some() {
                    let delete = CsrfDelete::Key(key);
                    driver.csrf_delete(&delete).map_err(CoreError::Driver)?;
                }
                Ok(csrf)
            })
    }

    /// Delete many CSRF key, value pairs that have expired using.
    fn delete_by_ttl(driver: &dyn Driver) -> CoreResult<usize> {
        let now = Utc::now();
        let delete = CsrfDelete::Ttl(now);
        match driver.csrf_delete(&delete) {
            Ok(count) => Ok(count),
            Err(err) => {
                warn!("{}", CoreError::Driver(err));
                Ok(0)
            }
        }
    }
}
