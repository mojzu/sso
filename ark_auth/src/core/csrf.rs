use crate::core::{Csrf, Error, Service};
use crate::driver::Driver;
use chrono::Utc;
use time::Duration;

/// Create one CSRF key, value pair with time to live in seconds. Key must be unique.
pub fn create(
    driver: &dyn Driver,
    service: &Service,
    key: &str,
    value: &str,
    ttl: i64,
) -> Result<Csrf, Error> {
    delete_by_ttl(driver)?;

    let ttl = Utc::now() + Duration::seconds(ttl);
    driver
        .csrf_create(key, value, &ttl, &service.id)
        .map_err(Error::Driver)
}

/// Read one CSRF key, value pair. CSRF key, value pair is deleted after one read.
pub fn read_by_key(driver: &dyn Driver, key: &str) -> Result<Option<Csrf>, Error> {
    delete_by_ttl(driver)?;

    driver
        .csrf_read_by_key(key)
        .map_err(Error::Driver)
        .and_then(|csrf| {
            if csrf.is_some() {
                driver.csrf_delete_by_key(key).map_err(Error::Driver)?;
            }
            Ok(csrf)
        })
}

/// Delete many CSRF key, value pairs that have expired using.
fn delete_by_ttl(driver: &dyn Driver) -> Result<usize, Error> {
    let now = Utc::now();
    match driver.csrf_delete_by_ttl(&now) {
        Ok(count) => Ok(count),
        Err(err) => {
            warn!("{}", Error::Driver(err));
            Ok(0)
        }
    }
}
