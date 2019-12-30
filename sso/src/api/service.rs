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
