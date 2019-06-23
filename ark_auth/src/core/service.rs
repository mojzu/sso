use crate::core::{Error, Service};
use crate::driver;
use url::Url;

/// List services where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    lt: &str,
    limit: i64,
) -> Result<Vec<String>, Error> {
    driver
        .service_list_where_id_lt(lt, limit)
        .map_err(Error::Driver)
}

/// List services where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    gt: &str,
    limit: i64,
) -> Result<Vec<String>, Error> {
    driver
        .service_list_where_id_gt(gt, limit)
        .map_err(Error::Driver)
}

/// Create service.
pub fn create(
    driver: &driver::Driver,
    is_enabled: bool,
    name: &str,
    url: &str,
) -> Result<Service, Error> {
    Url::parse(url).map_err(|_err| Error::BadRequest)?;
    driver
        .service_create(is_enabled, name, url)
        .map_err(Error::Driver)
}

/// Read service by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: &str,
) -> Result<Option<Service>, Error> {
    driver.service_read_by_id(id).map_err(Error::Driver)
}

/// Update service by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: &str,
    is_enabled: Option<bool>,
    name: Option<&str>,
) -> Result<Service, Error> {
    driver
        .service_update_by_id(id, is_enabled, name)
        .map_err(Error::Driver)
}

/// Delete service by ID.
pub fn delete_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: &str,
) -> Result<usize, Error> {
    driver.service_delete_by_id(id).map_err(Error::Driver)
}
