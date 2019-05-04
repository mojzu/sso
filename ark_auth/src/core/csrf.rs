use crate::core::{Csrf, Error, Service};
use crate::driver;

/// Create CSRF.
pub fn create(
    driver: &driver::Driver,
    service: &Service,
    key: &str,
    value: &str,
) -> Result<Csrf, Error> {
    unimplemented!();
}

/// Read CSRF by key.
pub fn read_by_key(driver: &driver::Driver, key: &str) -> Result<Csrf, Error> {
    unimplemented!();
}
