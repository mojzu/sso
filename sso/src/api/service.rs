use crate::{
    api::{
        result_audit_diff, result_audit_err, result_audit_subject, validate, ApiResult,
        ValidateRequest, ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Driver, Service, ServiceCreate, ServiceListFilter,
    ServiceListQuery, ServiceRead, ServiceUpdate, DEFAULT_LIMIT,
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
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
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

    pub user_allow_register: Option<bool>,

    #[validate(custom = "validate::text")]
    pub user_email_text: Option<String>,

    #[validate(url)]
    pub provider_local_url: Option<String>,

    #[validate(url)]
    pub provider_github_oauth2_url: Option<String>,

    #[validate(url)]
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl ValidateRequest<ServiceCreateRequest> for ServiceCreateRequest {}

impl From<ServiceCreateRequest> for ServiceCreate {
    fn from(request: ServiceCreateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            url: request.url,
            user_allow_register: request.user_allow_register.unwrap_or(false),
            user_email_text: request.user_email_text.unwrap_or_else(|| "".to_owned()),
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

    pub user_allow_register: Option<bool>,

    #[validate(custom = "validate::text")]
    pub user_email_text: Option<String>,

    #[validate(url)]
    pub provider_local_url: Option<String>,

    #[validate(url)]
    pub provider_github_oauth2_url: Option<String>,

    #[validate(url)]
    pub provider_microsoft_oauth2_url: Option<String>,
}

impl ValidateRequest<ServiceUpdateRequest> for ServiceUpdateRequest {}

impl Default for ServiceUpdateRequest {
    fn default() -> Self {
        Self {
            is_enabled: None,
            name: None,
            url: None,
            user_allow_register: None,
            user_email_text: None,
            provider_local_url: None,
            provider_github_oauth2_url: None,
            provider_microsoft_oauth2_url: None,
        }
    }
}

impl ServiceUpdateRequest {
    pub fn set_is_enabled(mut self, is_enabled: bool) -> Self {
        self.is_enabled = Some(is_enabled);
        self
    }

    pub fn set_user_allow_register(mut self, user_allow_register: bool) -> Self {
        self.user_allow_register = Some(user_allow_register);
        self
    }
}

impl From<ServiceUpdateRequest> for ServiceUpdate {
    fn from(request: ServiceUpdateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            url: request.url,
            user_allow_register: request.user_allow_register,
            user_email_text: request.user_email_text,
            provider_local_url: request.provider_local_url,
            provider_github_oauth2_url: request.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: request.provider_microsoft_oauth2_url,
        }
    }
}
