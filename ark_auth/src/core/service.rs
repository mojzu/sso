use crate::core::{Error, Service};
use crate::driver;

/// List services where ID is less than.
pub fn list_where_id_lt(
    driver: &driver::Driver,
    lt: i64,
    limit: i64,
) -> Result<Vec<Service>, Error> {
    driver
        .service_list_where_id_lt(lt, limit)
        .map_err(Error::Driver)
}

/// List services where ID is greater than.
pub fn list_where_id_gt(
    driver: &driver::Driver,
    gt: i64,
    limit: i64,
) -> Result<Vec<Service>, Error> {
    driver
        .service_list_where_id_gt(gt, limit)
        .map_err(Error::Driver)
}

/// Create service.
pub fn create(driver: &driver::Driver, name: &str, url: &str) -> Result<Service, Error> {
    driver.service_create(name, url).map_err(Error::Driver)
}

/// Read service by ID.
pub fn read_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: i64,
) -> Result<Option<Service>, Error> {
    driver.service_read_by_id(id).map_err(Error::Driver)
}

/// Update service by ID.
pub fn update_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: i64,
    name: Option<&str>,
) -> Result<Service, Error> {
    driver.service_update_by_id(id, name).map_err(Error::Driver)
}

/// Delete service by ID.
pub fn delete_by_id(
    driver: &driver::Driver,
    _service_mask: Option<&Service>,
    id: i64,
) -> Result<usize, Error> {
    driver.service_delete_by_id(id).map_err(Error::Driver)
}
