use crate::{
    api::{
        result_audit_diff, result_audit_err, result_audit_subject, validate, ApiError, ApiResult,
        ValidateRequest, ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Driver, User, UserCreate, UserListFilter, UserListQuery,
    UserPasswordMeta, UserRead, UserUpdate, DEFAULT_LIMIT,
};
use uuid::Uuid;
use validator::Validate;

// TODO(fix): This can produce URLs that are too long causing reqwest panics.

#[derive(Debug, Serialize, Deserialize, Validate, Builder)]
#[serde(deny_unknown_fields)]
pub struct UserListRequest {
    #[builder(default = "None")]
    gt: Option<Uuid>,

    #[builder(default = "None")]
    lt: Option<Uuid>,

    #[builder(default = "None")]
    #[validate(custom = "validate::name")]
    name_ge: Option<String>,

    #[builder(default = "None")]
    #[validate(custom = "validate::name")]
    name_le: Option<String>,

    #[builder(default = "None")]
    limit: Option<i64>,

    #[builder(default = "None")]
    offset_id: Option<Uuid>,

    #[builder(default = "None")]
    id: Option<Vec<Uuid>>,

    #[builder(default = "None")]
    email: Option<Vec<String>>,
}

impl ValidateRequest<UserListRequest> for UserListRequest {}
impl ValidateRequestQuery<UserListRequest> for UserListRequest {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserListResponse {
    pub meta: UserListRequest,
    pub data: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreateResponse {
    pub meta: UserPasswordMeta,
    pub data: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReadResponse {
    pub data: User,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserUpdateRequest {
    pub is_enabled: Option<bool>,
    #[validate(custom = "validate::name")]
    pub name: Option<String>,
    #[validate(custom = "validate::locale")]
    pub locale: Option<String>,
    #[validate(custom = "validate::timezone")]
    pub timezone: Option<String>,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
}

impl ValidateRequest<UserUpdateRequest> for UserUpdateRequest {}
