use crate::{
    api::{result_audit, result_audit_diff, validate, ValidateRequest, ValidateRequestQuery},
    AuditMeta, AuditType, Core, CoreError, CoreResult, Driver, Key, Service, ServiceCreate,
    ServiceListFilter, ServiceListQuery, ServiceUpdate,
};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct ServiceListRequest {
    #[builder(default = "None")]
    gt: Option<Uuid>,
    #[builder(default = "None")]
    lt: Option<Uuid>,
    #[builder(default = "None")]
    #[validate(custom = "validate::limit")]
    limit: Option<i64>,
    #[builder(default = "None")]
    id: Option<Vec<Uuid>>,
    #[builder(default = "None")]
    is_enabled: Option<bool>,
}

impl ValidateRequest<ServiceListRequest> for ServiceListRequest {}
impl ValidateRequestQuery<ServiceListRequest> for ServiceListRequest {}

impl ServiceListRequest {
    pub fn into_query_filter(self) -> (ServiceListQuery, ServiceListFilter) {
        let limit = self.limit.unwrap_or_else(Core::default_limit);
        let query = match (self.gt, self.lt) {
            (Some(gt), Some(_lt)) => ServiceListQuery::IdGt(gt, limit),
            (Some(gt), None) => ServiceListQuery::IdGt(gt, limit),
            (None, Some(lt)) => ServiceListQuery::IdLt(lt, limit),
            (None, None) => ServiceListQuery::Limit(limit),
        };

        let filter = ServiceListFilter {
            id: self.id,
            is_enabled: self.is_enabled,
        };

        (query, filter)
    }

    pub fn from_query_filter(query: ServiceListQuery, filter: ServiceListFilter) -> Self {
        match query {
            ServiceListQuery::Limit(limit) => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                id: filter.id,
                is_enabled: filter.is_enabled,
            },
            ServiceListQuery::IdGt(gt, limit) => Self {
                gt: Some(gt),
                lt: None,
                limit: Some(limit),
                id: filter.id,
                is_enabled: filter.is_enabled,
            },
            ServiceListQuery::IdLt(lt, limit) => Self {
                gt: None,
                lt: Some(lt),
                limit: Some(limit),
                id: filter.id,
                is_enabled: filter.is_enabled,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceListResponse {
    pub meta: ServiceListRequest,
    pub data: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceCreateRequest {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(url)]
    pub url: String,
    #[validate(url)]
    pub provider_local_url: Option<String>,
    #[validate(url)]
    pub provider_github_oauth2_url: Option<String>,
    #[validate(url)]
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl ValidateRequest<ServiceCreateRequest> for ServiceCreateRequest {}

impl ServiceCreateRequest {
    pub fn new<S1, S2>(is_enabled: bool, name: S1, url: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            url: url.into(),
            provider_local_url: None,
            provider_github_oauth2_url: None,
            provider_microsoft_oauth2_url: None,
        }
    }

    pub fn provider_local_url<S: Into<String>>(mut self, provider_local_url: S) -> Self {
        self.provider_local_url = Some(provider_local_url.into());
        self
    }

    pub fn provider_github_oauth2_url<S: Into<String>>(
        mut self,
        provider_github_oauth2_url: S,
    ) -> Self {
        self.provider_github_oauth2_url = Some(provider_github_oauth2_url.into());
        self
    }

    pub fn provider_microsoft_oauth2_url<S: Into<String>>(
        mut self,
        provider_microsoft_oauth2_url: S,
    ) -> Self {
        self.provider_microsoft_oauth2_url = Some(provider_microsoft_oauth2_url.into());
        self
    }
}

impl From<ServiceCreateRequest> for ServiceCreate {
    fn from(request: ServiceCreateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            url: request.url,
            provider_local_url: request.provider_local_url,
            provider_github_oauth2_url: request.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: request.provider_microsoft_oauth2_url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceReadResponse {
    pub data: Service,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct ServiceUpdateRequest {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
    #[validate(url)]
    pub url: Option<String>,
    #[validate(url)]
    pub provider_local_url: Option<String>,
    #[validate(url)]
    pub provider_github_oauth2_url: Option<String>,
    #[validate(url)]
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl ValidateRequest<ServiceUpdateRequest> for ServiceUpdateRequest {}

impl From<ServiceUpdateRequest> for ServiceUpdate {
    fn from(request: ServiceUpdateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            url: request.url,
            provider_local_url: request.provider_local_url,
            provider_github_oauth2_url: request.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: request.provider_microsoft_oauth2_url,
        }
    }
}

pub fn service_list(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: ServiceListRequest,
) -> CoreResult<ServiceListResponse> {
    ServiceListRequest::api_validate(&request)?;
    let audit_type = AuditType::ServiceList;
    let _audit = Key::authenticate_root(driver, audit_meta, key_value, audit_type)?;

    let (query, filter) = request.into_query_filter();
    Service::list(driver, &query, &filter).map(|data| ServiceListResponse {
        meta: ServiceListRequest::from_query_filter(query, filter),
        data,
    })
}

pub fn service_create(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    request: ServiceCreateRequest,
) -> CoreResult<ServiceReadResponse> {
    ServiceCreateRequest::api_validate(&request)?;
    let audit_type = AuditType::ServiceCreate;
    let mut audit = Key::authenticate_root(driver, audit_meta, key_value, audit_type)?;

    let create: ServiceCreate = request.into();
    let res = Service::create(driver, &create);
    result_audit(driver, res, &mut audit, audit_type).map(|data| ServiceReadResponse { data })
}

pub fn service_read(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    service_id: Uuid,
) -> CoreResult<ServiceReadResponse> {
    let audit_type = AuditType::ServiceRead;
    let (service, _audit) = Key::authenticate(driver, audit_meta, key_value, audit_type)?;

    Service::read_opt(driver, service.as_ref(), &service_id)
        .and_then(|service| service.ok_or_else(|| CoreError::NotFound))
        .map(|data| ServiceReadResponse { data })
}

pub fn service_update(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    service_id: Uuid,
    request: ServiceUpdateRequest,
) -> CoreResult<ServiceReadResponse> {
    ServiceUpdateRequest::api_validate(&request)?;
    let audit_type = AuditType::ServiceUpdate;
    let (service, mut audit) = Key::authenticate(driver, audit_meta, key_value, audit_type)?;

    let res = Service::read_opt(driver, service.as_ref(), &service_id)
        .and_then(|service| service.ok_or_else(|| CoreError::NotFound))
        .and_then(|previous_service| {
            let update: ServiceUpdate = request.into();
            Service::update(driver, service.as_ref(), service_id, &update)
                .map(|next_service| (previous_service, next_service))
        });
    result_audit_diff(driver, res, &mut audit, audit_type).map(|data| ServiceReadResponse { data })
}

pub fn service_delete(
    driver: &dyn Driver,
    key_value: Option<String>,
    audit_meta: AuditMeta,
    service_id: Uuid,
) -> CoreResult<()> {
    let audit_type = AuditType::ServiceDelete;
    let (service, mut audit) = Key::authenticate(driver, audit_meta, key_value, audit_type)?;

    let res = Service::read_opt(driver, service.as_ref(), &service_id)
        .and_then(|service| service.ok_or_else(|| CoreError::NotFound))
        .and_then(|previous_service| {
            Service::delete(driver, service.as_ref(), service_id).map(|_| previous_service)
        });
    result_audit(driver, res, &mut audit, audit_type).map(|_| ())
}
