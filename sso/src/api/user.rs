use crate::{
    api::{
        result_audit_diff, result_audit_err, result_audit_subject, validate, ApiResult,
        ValidateRequest, ValidateRequestQuery,
    },
    AuditBuilder, AuditMeta, AuditType, Driver, User, UserCreate, UserListFilter, UserListQuery,
    UserPasswordMeta, UserRead, UserUpdate, DEFAULT_LIMIT,
};
use uuid::Uuid;
use validator::Validate;

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
    #[validate(custom = "validate::limit")]
    limit: Option<i64>,

    #[builder(default = "None")]
    offset_id: Option<Uuid>,

    #[builder(default = "None")]
    id: Option<Vec<Uuid>>,

    #[builder(default = "None")]
    #[validate(custom = "validate::email_vec")]
    email: Option<Vec<String>>,
}

impl ValidateRequest<UserListRequest> for UserListRequest {}
impl ValidateRequestQuery<UserListRequest> for UserListRequest {}

impl UserListRequest {
    pub fn into_query_filter(self) -> (UserListQuery, UserListFilter) {
        let limit = self.limit.unwrap_or(DEFAULT_LIMIT);
        let query = match (self.gt, self.lt) {
            (Some(gt), Some(_lt)) => UserListQuery::IdGt(gt, limit),
            (Some(gt), None) => UserListQuery::IdGt(gt, limit),
            (None, Some(lt)) => UserListQuery::IdLt(lt, limit),
            (None, None) => UserListQuery::Limit(limit),
        };
        // TODO(refactor): Support user name sorting.

        let filter = UserListFilter {
            id: self.id,
            email: self.email,
        };

        (query, filter)
    }

    pub fn from_query_filter(query: UserListQuery, filter: UserListFilter) -> Self {
        let mut builder = UserListRequestBuilder::default();
        builder.id(filter.id);
        builder.email(filter.email);
        match query {
            UserListQuery::Limit(limit) => builder.limit(Some(limit)).build().unwrap(),
            UserListQuery::IdGt(gt, limit) => {
                builder.gt(Some(gt)).limit(Some(limit)).build().unwrap()
            }
            UserListQuery::IdLt(lt, limit) => {
                builder.lt(Some(lt)).limit(Some(limit)).build().unwrap()
            }
            UserListQuery::NameGe(name_ge, limit, offset_id) => builder
                .name_ge(Some(name_ge))
                .limit(Some(limit))
                .offset_id(offset_id)
                .build()
                .unwrap(),
            UserListQuery::NameLe(name_le, limit, offset_id) => builder
                .name_le(Some(name_le))
                .limit(Some(limit))
                .offset_id(offset_id)
                .build()
                .unwrap(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserListResponse {
    pub meta: UserListRequest,
    pub data: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct UserCreateRequest {
    pub is_enabled: bool,
    #[validate(custom = "validate::name")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom = "validate::locale")]
    pub locale: String,
    #[validate(custom = "validate::timezone")]
    pub timezone: String,
    pub password_allow_reset: Option<bool>,
    pub password_require_update: Option<bool>,
    #[validate(custom = "validate::password")]
    pub password: Option<String>,
}

impl ValidateRequest<UserCreateRequest> for UserCreateRequest {}

impl UserCreateRequest {
    pub fn new<S1, S2, S3, S4>(
        is_enabled: bool,
        name: S1,
        email: S2,
        locale: S3,
        timezone: S4,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        Self {
            is_enabled,
            name: name.into(),
            email: email.into(),
            locale: locale.into(),
            timezone: timezone.into(),
            password_allow_reset: None,
            password_require_update: None,
            password: None,
        }
    }

    pub fn with_password<S1: Into<String>>(
        mut self,
        password_allow_reset: bool,
        password_require_update: bool,
        password: S1,
    ) -> Self {
        self.password_allow_reset = Some(password_allow_reset);
        self.password_require_update = Some(password_require_update);
        self.password = Some(password.into());
        self
    }
}

impl From<UserCreateRequest> for UserCreate {
    fn from(request: UserCreateRequest) -> Self {
        Self {
            is_enabled: request.is_enabled,
            name: request.name,
            email: request.email,
            locale: request.locale,
            timezone: request.timezone,
            password_allow_reset: request.password_allow_reset.unwrap_or(false),
            password_require_update: request.password_require_update.unwrap_or(false),
            password_hash: request.password,
        }
    }
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

impl Default for UserUpdateRequest {
    fn default() -> Self {
        Self {
            is_enabled: None,
            name: None,
            locale: None,
            timezone: None,
            password_allow_reset: None,
            password_require_update: None,
        }
    }
}

impl UserUpdateRequest {
    pub fn is_enabled(mut self, is_enabled: bool) -> Self {
        self.is_enabled = Some(is_enabled);
        self
    }
}

pub fn user_list(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    request: UserListRequest,
) -> ApiResult<UserListResponse> {
    UserListRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserList);
    let (query, filter) = request.into_query_filter();

    let res = server_user::list(driver, &mut audit, key_value, &query, &filter);
    result_audit_err(driver, &audit, res).map(|data| UserListResponse {
        meta: UserListRequest::from_query_filter(query, filter),
        data,
    })
}

pub fn user_create(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    password_meta: UserPasswordMeta,
    request: UserCreateRequest,
) -> ApiResult<UserCreateResponse> {
    UserCreateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserCreate);
    let create: UserCreate = request.into();

    let res = server_user::create(driver, &mut audit, key_value, create);
    result_audit_subject(driver, &audit, res).map(|data| UserCreateResponse {
        meta: password_meta,
        data,
    })
}

pub fn user_read(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    user_id: Uuid,
) -> ApiResult<UserReadResponse> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserRead);
    let read = UserRead::Id(user_id);

    let res = server_user::read(driver, &mut audit, key_value, &read);
    result_audit_err(driver, &audit, res).map(|data| UserReadResponse { data })
}

pub fn user_update(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    user_id: Uuid,
    request: UserUpdateRequest,
) -> ApiResult<UserReadResponse> {
    UserUpdateRequest::api_validate(&request)?;
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserUpdate);
    let update = UserUpdate {
        is_enabled: request.is_enabled,
        name: request.name,
        locale: request.locale,
        timezone: request.timezone,
        password_allow_reset: request.password_allow_reset,
        password_require_update: request.password_require_update,
    };

    let res = server_user::update(driver, &mut audit, key_value, user_id, update);
    result_audit_diff(driver, &audit, res).map(|data| UserReadResponse { data })
}

pub fn user_delete(
    driver: &dyn Driver,
    audit_meta: AuditMeta,
    key_value: Option<String>,
    user_id: Uuid,
) -> ApiResult<()> {
    let mut audit = AuditBuilder::new(audit_meta, AuditType::UserDelete);

    let res = server_user::delete(driver, &mut audit, key_value, user_id);
    result_audit_subject(driver, &audit, res).map(|_| ())
}

mod server_user {
    use super::*;
    use crate::{
        api::{ApiError, ApiResult},
        AuditBuilder, Auth, CoreError, Driver, Service, User, UserList, UserListFilter,
        UserListQuery, UserRead,
    };

    pub fn list(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        query: &UserListQuery,
        filter: &UserListFilter,
    ) -> ApiResult<Vec<User>> {
        let _service =
            Auth::authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let list = UserList { query, filter };
        driver
            .user_list(&list)
            .map_err(CoreError::Driver)
            .map_err(ApiError::BadRequest)
    }

    pub fn create(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        mut create: UserCreate,
    ) -> ApiResult<User> {
        let service =
            Auth::authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        User::create(driver, service.as_ref(), &mut create).map_err(ApiError::BadRequest)
    }

    pub fn read(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        read: &UserRead,
    ) -> ApiResult<User> {
        let service =
            Auth::authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        read_inner(driver, service.as_ref(), read)
    }

    pub fn update(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        user_id: Uuid,
        update: UserUpdate,
    ) -> ApiResult<(User, User)> {
        let service =
            Auth::authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let read = UserRead::Id(user_id);
        let previous_user = read_inner(driver, service.as_ref(), &read)?;
        let user = User::update(driver, service.as_ref(), user_id, &update)
            .map_err(ApiError::BadRequest)?;
        Ok((previous_user, user))
    }

    pub fn delete(
        driver: &dyn Driver,
        audit: &mut AuditBuilder,
        key_value: Option<String>,
        user_id: Uuid,
    ) -> ApiResult<User> {
        let service =
            Auth::authenticate(driver, audit, key_value).map_err(ApiError::Unauthorised)?;

        let read = UserRead::Id(user_id);
        let user = read_inner(driver, service.as_ref(), &read)?;
        driver
            .user_delete(&user_id)
            .map_err(CoreError::Driver)
            .map_err(ApiError::BadRequest)
            .map(|_| user)
    }

    fn read_inner(
        driver: &dyn Driver,
        service: Option<&Service>,
        read: &UserRead,
    ) -> ApiResult<User> {
        User::read(driver, service, read).map_err(ApiError::NotFound)
    }
}
