use crate::{AuditBuilder, CoreError, CoreResult, CoreUtil, Driver};
use chrono::{DateTime, Utc};
use serde::ser::Serialize;
use std::fmt;
use url::Url;
use uuid::Uuid;

/// Service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub id: Uuid,
    pub is_enabled: bool,
    pub name: String,
    pub url: String,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service {}", self.id)?;
        write!(f, "\n\tcreated_at {}", self.created_at)?;
        write!(f, "\n\tupdated_at {}", self.updated_at)?;
        write!(f, "\n\tis_enabled {}", self.is_enabled)?;
        write!(f, "\n\tname {}", self.name)?;
        write!(f, "\n\turl {}", self.url)
    }
}

/// Service list.
#[derive(Debug)]
pub enum ServiceList {
    Limit(i64),
    IdGt(Uuid, i64),
    IdLt(Uuid, i64),
}

/// Service create data.
pub struct ServiceCreate {
    pub is_enabled: bool,
    pub name: String,
    pub url: String,
}

/// Service update data.
pub struct ServiceUpdate {
    pub is_enabled: Option<bool>,
    pub name: Option<String>,
}

/// Service callback URL query.
#[derive(Serialize, Deserialize)]
struct ServiceCallbackQuery<S: Serialize> {
    #[serde(rename = "type")]
    type_: String,
    #[serde(flatten)]
    data: S,
}

impl<S: Serialize> ServiceCallbackQuery<S> {
    pub fn new<T: Into<String>>(type_: T, data: S) -> Self {
        Self {
            type_: type_.into(),
            data,
        }
    }
}

impl Service {
    pub fn callback_url<S: Serialize>(&self, type_: &str, data: S) -> CoreResult<Url> {
        let mut url = Url::parse(&self.url).unwrap();
        let query = ServiceCallbackQuery::new(type_, data);
        let query = CoreUtil::qs_ser(&query)?;
        url.set_query(Some(&query));
        Ok(url)
    }

    /// List services using query.
    pub fn list(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        list: &ServiceList,
    ) -> CoreResult<Vec<Service>> {
        driver.service_list(list).map_err(CoreError::Driver)
    }

    /// Create service.
    pub fn create(
        driver: &dyn Driver,
        _audit: &mut AuditBuilder,
        is_enabled: bool,
        name: String,
        url: String,
    ) -> CoreResult<Service> {
        Url::parse(&url).map_err(|_err| CoreError::BadRequest)?;
        let create = ServiceCreate {
            is_enabled,
            name,
            url,
        };
        driver.service_create(&create).map_err(CoreError::Driver)
    }

    /// Read service (optional).
    pub fn read_opt(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<Option<Service>> {
        driver.service_read_opt(&id).map_err(CoreError::Driver)
    }

    /// Update service by ID.
    pub fn update(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
        is_enabled: Option<bool>,
        name: Option<String>,
    ) -> CoreResult<Service> {
        let update = ServiceUpdate { is_enabled, name };
        driver
            .service_update(&id, &update)
            .map_err(CoreError::Driver)
    }

    /// Delete service by ID.
    pub fn delete(
        driver: &dyn Driver,
        _service_mask: Option<&Service>,
        _audit: &mut AuditBuilder,
        id: Uuid,
    ) -> CoreResult<usize> {
        driver.service_delete(&id).map_err(CoreError::Driver)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[derive(Serialize)]
    struct CallbackData {
        email: String,
        token: String,
    }

    #[test]
    fn builds_service_callback_url() {
        let id = "6a9c6cfb7e15498b99e057153f0a212b";
        let id = Uuid::parse_str(id).unwrap();
        let service = Service {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            id,
            is_enabled: true,
            name: "Service Name".to_owned(),
            url: "http://localhost:9000".to_owned(),
        };
        let callback_data = CallbackData {
            email: "user@test.com".to_owned(),
            token: "6a9c6cfb7e15498b99e057153f0a212b".to_owned(),
        };
        let url = service
            .callback_url("reset_password", &callback_data)
            .unwrap();
        assert_eq!(
            url.to_string(),
            "http://localhost:9000/?type=reset_password&email=user%40test.com&token=6a9c6cfb7e15498b99e057153f0a212b"
        );
    }
}
