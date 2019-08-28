use crate::{
    core::{audit::AuditBuilder, Error, Service, ServiceQuery, DEFAULT_LIMIT},
    driver::Driver,
};
use url::Url;
use uuid::Uuid;

/// List services using query.
pub fn list(
    driver: &dyn Driver,
    _audit: &mut AuditBuilder,
    query: &ServiceQuery,
) -> Result<Vec<Uuid>, Error> {
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT);

    match &query.gt {
        Some(gt) => driver
            .service_list_where_id_gt(*gt, limit)
            .map_err(Error::Driver),
        None => match &query.lt {
            Some(lt) => driver
                .service_list_where_id_lt(*lt, limit)
                .map_err(Error::Driver),
            None => driver
                .service_list_where_id_gt(Uuid::nil(), limit)
                .map_err(Error::Driver),
        },
    }
}

/// Create service.
pub fn create(
    driver: &dyn Driver,
    _audit: &mut AuditBuilder,
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
    driver: &dyn Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: Uuid,
) -> Result<Option<Service>, Error> {
    driver.service_read_by_id(id).map_err(Error::Driver)
}

/// Update service by ID.
pub fn update_by_id(
    driver: &dyn Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: Uuid,
    is_enabled: Option<bool>,
    name: Option<&str>,
) -> Result<Service, Error> {
    driver
        .service_update_by_id(id, is_enabled, name)
        .map_err(Error::Driver)
}

/// Delete service by ID.
pub fn delete_by_id(
    driver: &dyn Driver,
    _service_mask: Option<&Service>,
    _audit: &mut AuditBuilder,
    id: Uuid,
) -> Result<usize, Error> {
    driver.service_delete_by_id(id).map_err(Error::Driver)
}
