use crate::{
    core::{Csrf, Error, Service},
    driver,
};
use chrono::Utc;
use time::Duration;

/// Create one CSRF key, value pair. Key must be unique.
pub fn create(
    driver: &driver::Driver,
    service: &Service,
    key: &str,
    value: &str,
) -> Result<Csrf, Error> {
    delete_by_age(driver)?;

    driver
        .csrf_create(key, value, service.id)
        .map_err(Error::Driver)
}

/// Read one CSRF key, value pair. CSRF key, value pair is deleted after one read.
pub fn read_by_key(driver: &driver::Driver, key: &str) -> Result<Option<Csrf>, Error> {
    delete_by_age(driver)?;

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

/// Delete many CSRF key, value pairs created more than one hour ago.
fn delete_by_age(driver: &driver::Driver) -> Result<usize, Error> {
    let previous_hour = Utc::now() - Duration::hours(1);

    match driver.csrf_delete_by_created_at(&previous_hour) {
        Ok(count) => Ok(count),
        Err(err) => {
            warn!("{}", Error::Driver(err));
            Ok(0)
        }
    }
}
