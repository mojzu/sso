use crate::{CoreError, CoreResult, Driver, Service};
use chrono::{DateTime, Utc};
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

impl Csrf {
    /// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
    pub fn create(
        driver: &dyn Driver,
        service: &Service,
        key: &str,
        value: &str,
        ttl: i64,
    ) -> CoreResult<Csrf> {
        Csrf::delete_by_ttl(driver)?;

        let ttl = Utc::now() + Duration::seconds(ttl);
        driver
            .csrf_create(key, value, &ttl, service.id)
            .map_err(CoreError::Driver)
    }

    /// Read one CSRF key, value pair. CSRF key, value pair is deleted after one read.
    pub fn read_by_key(driver: &dyn Driver, key: &str) -> CoreResult<Option<Csrf>> {
        Csrf::delete_by_ttl(driver)?;

        driver
            .csrf_read_by_key(key)
            .map_err(CoreError::Driver)
            .and_then(|csrf| {
                if csrf.is_some() {
                    driver.csrf_delete_by_key(key).map_err(CoreError::Driver)?;
                }
                Ok(csrf)
            })
    }

    /// Delete many CSRF key, value pairs that have expired using.
    fn delete_by_ttl(driver: &dyn Driver) -> CoreResult<usize> {
        let now = Utc::now();
        match driver.csrf_delete_by_ttl(&now) {
            Ok(count) => Ok(count),
            Err(err) => {
                warn!("{}", CoreError::Driver(err));
                Ok(0)
            }
        }
    }
}
