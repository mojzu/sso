use crate::{
    grpc::{self, pb},
    *,
};
use chrono::{DateTime, Utc};
use core::pin::Pin;
use std::future::Future;
use std::net::SocketAddr;
use tonic::{metadata::MetadataMap, Request, Response, Status};
use uuid::Uuid;

/// Run a blocking closure on threadpool.
pub fn blocking<T, E, F>(f: F) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>
where
    F: Send + FnOnce() -> Result<T, E> + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    // TODO(refactor): Improve error handling.
    let mut f = Some(f);
    let fut = async move { tokio_executor::blocking::run(move || (f.take().unwrap())()).await };
    Box::pin(fut)
}

/// Method errors.
#[derive(Debug, Fail)]
pub enum MethodError {
    #[fail(display = "BadRequest {}", _0)]
    BadRequest(#[fail(cause)] DriverError),

    #[fail(display = "Unauthorised {}", _0)]
    Unauthorised(#[fail(cause)] DriverError),

    #[fail(display = "Forbidden {}", _0)]
    Forbidden(#[fail(cause)] DriverError),

    #[fail(display = "NotFound {}", _0)]
    NotFound(#[fail(cause)] DriverError),

    #[fail(display = "InternalServerError {}", _0)]
    InternalServerError(#[fail(cause)] DriverError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MethodErrorData {
    code: u16,
    message: String,
}

impl MethodError {
    fn get_status(&self) -> Status {
        match self {
            MethodError::BadRequest(e) => Status::invalid_argument(format!("{}", e)),
            MethodError::Unauthorised(e) => Status::unauthenticated(format!("{}", e)),
            MethodError::Forbidden(e) => Status::permission_denied(format!("{}", e)),
            MethodError::NotFound(e) => Status::not_found(format!("{}", e)),
            MethodError::InternalServerError(e) => Status::internal(format!("{}", e)),
        }
    }

    fn get_data(&self) -> MethodErrorData {
        let e = self.get_status();
        MethodErrorData {
            code: e.code() as u16,
            message: e.message().to_owned(),
        }
    }
}

impl From<MethodError> for Status {
    fn from(e: MethodError) -> Self {
        e.get_status()
    }
}

/// Method result wrapper type.
pub type MethodResult<T> = Result<T, MethodError>;

/// Request message with extracted metadata.
#[derive(Debug)]
pub struct MethodRequest<T> {
    audit: AuditMeta,
    auth: Option<String>,
    message: T,
}

impl<T> MethodRequest<T> {
    pub fn from_request<R>(request: Request<R>) -> MethodResult<Self>
    where
        R: validator::Validate,
        T: From<R>,
    {
        let (audit, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
        let message = grpc::validate::validate(request.into_inner())?;
        Ok(MethodRequest {
            audit,
            auth,
            message: message.into(),
        })
    }

    pub fn from_unit(request: Request<()>) -> MethodResult<Self>
    where
        T: Default,
    {
        let (audit, auth) = request_audit_auth(request.remote_addr(), request.metadata())?;
        Ok(MethodRequest {
            audit,
            auth,
            message: T::default(),
        })
    }

    pub fn into_inner(self) -> (AuditMeta, Option<String>, T) {
        (self.audit, self.auth, self.message)
    }
}

/// Method response wrapper type.
pub type MethodResponse<T> = Result<Response<T>, MethodError>;

pub fn audit_result<T>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: MethodResult<T>,
) -> MethodResult<T> {
    match res {
        Ok(res) => {
            audit
                .create_data::<bool>(driver, 0, None, None)
                .map_err(MethodError::InternalServerError)?;
            Ok(res)
        }
        Err(e) => {
            let data = e.get_data();
            audit
                .create_data(
                    driver,
                    data.code,
                    None,
                    Some(AuditDiffBuilder::typed_data("error", data)),
                )
                .map_err(MethodError::InternalServerError)?;
            Err(e)
        }
    }
}

pub fn audit_result_err<T>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: MethodResult<T>,
) -> MethodResult<T> {
    match res {
        Ok(res) => Ok(res),
        Err(e) => {
            let data = e.get_data();
            audit
                .create_data(
                    driver,
                    data.code,
                    None,
                    Some(AuditDiffBuilder::typed_data("error", data)),
                )
                .map_err(MethodError::InternalServerError)?;
            Err(e)
        }
    }
}

pub fn audit_result_subject<T: AuditSubject>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: MethodResult<T>,
) -> MethodResult<T> {
    match res {
        Ok(res) => {
            audit
                .create_data::<bool>(driver, 0, Some(res.subject()), None)
                .map_err(MethodError::InternalServerError)?;
            Ok(res)
        }
        Err(e) => {
            let data = e.get_data();
            audit
                .create_data(
                    driver,
                    data.code,
                    None,
                    Some(AuditDiffBuilder::typed_data("error", data)),
                )
                .map_err(MethodError::InternalServerError)?;
            Err(e)
        }
    }
}

pub fn audit_result_diff<T: AuditSubject + AuditDiff>(
    driver: &dyn Driver,
    audit: &AuditBuilder,
    res: MethodResult<(T, T)>,
) -> MethodResult<T> {
    match res {
        Ok((p, n)) => {
            let diff = n.diff(&p);
            audit
                .create_data(driver, 0, Some(n.subject()), Some(diff))
                .map_err(MethodError::InternalServerError)?;
            Ok(n)
        }
        Err(e) => {
            let data = e.get_data();
            audit
                .create_data(
                    driver,
                    data.code,
                    None,
                    Some(AuditDiffBuilder::typed_data("error", data)),
                )
                .map_err(MethodError::InternalServerError)?;
            Err(e)
        }
    }
}

/// Get audit meta and authorisation header from request metadata.
pub fn request_audit_auth(
    remote: Option<SocketAddr>,
    metadata: &MetadataMap,
) -> MethodResult<(AuditMeta, Option<String>)> {
    let user_agent = match metadata.get("user-agent") {
        Some(value) => value.to_str().unwrap().to_owned(),
        None => String::from("none"),
    };
    let remote = match remote {
        Some(remote) => format!("{}", remote),
        None => String::from("unknown"),
    };
    let forwarded = match metadata.get("x-forwarded-for") {
        Some(value) => Some(value.to_str().unwrap().to_owned()),
        None => None,
    };
    let user = match metadata.get(HEADER_USER_AUTHORISATION_NAME) {
        Some(value) => {
            let u = value.to_str().unwrap();
            pattern::HeaderAuth::parse(u)
        }
        None => None,
    };
    let auth = match metadata.get(HEADER_AUTHORISATION_NAME) {
        Some(value) => Some(value.to_str().unwrap().to_owned()),
        None => None,
    };
    Ok((AuditMeta::new(user_agent, remote, forwarded, user), auth))
}

// TODO(refactor): Improve translation code between api/grpc.
// TODO(refactor): Unwrap check and cleanup.

pub fn timestamp_opt_to_datetime_opt(ti: Option<prost_types::Timestamp>) -> Option<DateTime<Utc>> {
    match ti {
        Some(ti) => {
            let st: Result<std::time::SystemTime, std::time::Duration> = ti.into();
            let dt: DateTime<Utc> = st.unwrap().into();
            Some(dt)
        }
        None => None,
    }
}

pub fn timestamp_opt_to_datetime(ti: Option<prost_types::Timestamp>) -> DateTime<Utc> {
    timestamp_opt_to_datetime_opt(ti).unwrap()
}

pub fn datetime_to_timestamp_opt(dt: DateTime<Utc>) -> Option<prost_types::Timestamp> {
    let st: std::time::SystemTime = dt.into();
    let ti: prost_types::Timestamp = st.into();
    Some(ti)
}

pub fn string_to_uuid(s: String) -> Uuid {
    Uuid::parse_str(s.as_ref()).unwrap()
}

pub fn string_opt_to_uuid_opt(s: Option<String>) -> Option<Uuid> {
    match s {
        Some(s) => {
            let u: Uuid = Uuid::parse_str(s.as_ref()).unwrap();
            Some(u)
        }
        None => None,
    }
}

pub fn string_vec_to_uuid_vec_opt(s: Vec<String>) -> Option<Vec<Uuid>> {
    if s.is_empty() {
        None
    } else {
        Some(
            s.into_iter()
                .map(|s| Uuid::parse_str(s.as_ref()).unwrap())
                .collect(),
        )
    }
}

pub fn string_vec_to_string_vec_opt(s: Vec<String>) -> Option<Vec<String>> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn i32_vec_to_key_type_vec_opt(s: Vec<i32>) -> Option<Vec<KeyType>> {
    if s.is_empty() {
        None
    } else {
        Some(s.into_iter().map(|s| KeyType::from_i32(s)).collect())
    }
}

pub fn key_type_vec_opt_to_i32_vec(s: Option<Vec<KeyType>>) -> Vec<i32> {
    match s {
        Some(s) => s.into_iter().map(|x| x as i32).collect(),
        None => Vec::new(),
    }
}

pub fn uuid_to_string(u: Uuid) -> String {
    format!("{}", u)
}

pub fn uuid_opt_to_string_opt(u: Option<Uuid>) -> Option<String> {
    match u {
        Some(u) => Some(uuid_to_string(u)),
        None => None,
    }
}

pub fn uuid_vec_opt_to_string_vec(u: Option<Vec<Uuid>>) -> Vec<String> {
    match u {
        Some(u) => u
            .into_iter()
            .map::<String, _>(|x| format!("{}", x))
            .collect(),
        None => Vec::new(),
    }
}

fn struct_kind_to_value(s: prost_types::value::Kind) -> serde_json::Value {
    match s {
        prost_types::value::Kind::NullValue(_x) => serde_json::Value::Null,
        prost_types::value::Kind::NumberValue(x) => {
            let n = serde_json::Number::from_f64(x);
            serde_json::Value::Number(n.unwrap())
        }
        prost_types::value::Kind::StringValue(x) => serde_json::Value::String(x),
        prost_types::value::Kind::BoolValue(x) => serde_json::Value::Bool(x),
        prost_types::value::Kind::StructValue(x) => struct_opt_to_value_opt(Some(x)).unwrap(),
        prost_types::value::Kind::ListValue(x) => {
            let mut v = Vec::new();
            for value in x.values {
                if let Some(kind) = value.kind {
                    v.push(struct_kind_to_value(kind))
                }
            }
            serde_json::Value::Array(v)
        }
    }
}

pub fn struct_opt_to_value_opt(s: Option<prost_types::Struct>) -> Option<serde_json::Value> {
    let mut m = serde_json::Map::default();
    match s {
        Some(s) => {
            for (key, value) in s.fields {
                if let Some(kind) = value.kind {
                    m.insert(key, struct_kind_to_value(kind));
                }
            }
            Some(serde_json::Value::Object(m))
        }
        None => None,
    }
}

fn value_to_struct_value(value: serde_json::Value) -> prost_types::Value {
    let kind: prost_types::value::Kind = match value {
        serde_json::Value::Null => prost_types::value::Kind::NullValue(0),
        serde_json::Value::Bool(x) => prost_types::value::Kind::BoolValue(x),
        serde_json::Value::Number(x) => prost_types::value::Kind::NumberValue(x.as_f64().unwrap()),
        serde_json::Value::String(x) => prost_types::value::Kind::StringValue(x),
        serde_json::Value::Array(x) => {
            let mut v = Vec::new();
            for value in x {
                v.push(value_to_struct_value(value));
            }
            prost_types::value::Kind::ListValue(prost_types::ListValue { values: v })
        }
        serde_json::Value::Object(x) => {
            let mut fields = std::collections::BTreeMap::new();
            for (key, value) in x {
                fields.insert(key, value_to_struct_value(value));
            }
            prost_types::value::Kind::StructValue(prost_types::Struct { fields })
        }
    };
    prost_types::Value { kind: Some(kind) }
}

pub fn value_to_struct_opt(s: serde_json::Value) -> Option<prost_types::Struct> {
    let mut fields = std::collections::BTreeMap::new();
    match s {
        serde_json::Value::Object(x) => {
            for (key, value) in x {
                fields.insert(key, value_to_struct_value(value));
            }
            Some(prost_types::Struct { fields })
        }
        _ => None,
    }
}

impl From<pb::KeyListRequest> for KeyList {
    fn from(r: pb::KeyListRequest) -> Self {
        let limit = r.limit.unwrap_or(DEFAULT_LIMIT);
        let gt = string_opt_to_uuid_opt(r.gt);
        let lt = string_opt_to_uuid_opt(r.lt);
        let query = match (gt, lt) {
            (Some(gt), Some(_lt)) => KeyListQuery::IdGt(gt),
            (Some(gt), None) => KeyListQuery::IdGt(gt),
            (None, Some(lt)) => KeyListQuery::IdLt(lt),
            (None, None) => KeyListQuery::Limit,
        };
        let filter = KeyListFilter {
            id: string_vec_to_uuid_vec_opt(r.id),
            is_enabled: r.is_enabled,
            is_revoked: r.is_revoked,
            type_: i32_vec_to_key_type_vec_opt(r.r#type),
            service_id: string_vec_to_uuid_vec_opt(r.service_id),
            user_id: string_vec_to_uuid_vec_opt(r.user_id),
            limit,
        };
        KeyList { query, filter }
    }
}

impl From<KeyList> for pb::KeyListRequest {
    fn from(l: KeyList) -> Self {
        let id = uuid_vec_opt_to_string_vec(l.filter.id);
        let is_enabled = l.filter.is_enabled;
        let is_revoked = l.filter.is_revoked;
        let type_ = key_type_vec_opt_to_i32_vec(l.filter.type_);
        let service_id = uuid_vec_opt_to_string_vec(l.filter.service_id);
        let user_id = uuid_vec_opt_to_string_vec(l.filter.user_id);
        let limit = l.filter.limit;
        match l.query {
            KeyListQuery::Limit => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                id,
                is_enabled,
                is_revoked,
                r#type: type_,
                service_id,
                user_id,
            },
            KeyListQuery::IdGt(gt) => Self {
                gt: Some(uuid_to_string(gt)),
                lt: None,
                limit: Some(limit),
                id,
                is_enabled,
                is_revoked,
                r#type: type_,
                service_id,
                user_id,
            },
            KeyListQuery::IdLt(lt) => Self {
                gt: None,
                lt: Some(uuid_to_string(lt)),
                limit: Some(limit),
                id,
                is_enabled,
                is_revoked,
                r#type: type_,
                service_id,
                user_id,
            },
        }
    }
}

impl From<Key> for pb::Key {
    fn from(r: Key) -> Self {
        Self {
            created_at: datetime_to_timestamp_opt(r.created_at),
            updated_at: datetime_to_timestamp_opt(r.updated_at),
            id: uuid_to_string(r.id),
            is_enabled: r.is_enabled,
            is_revoked: r.is_revoked,
            r#type: r.type_ as i32,
            name: r.name,
            service_id: uuid_opt_to_string_opt(r.service_id),
            user_id: uuid_opt_to_string_opt(r.user_id),
        }
    }
}

impl From<KeyWithValue> for pb::Key {
    fn from(r: KeyWithValue) -> Self {
        Self {
            created_at: datetime_to_timestamp_opt(r.created_at),
            updated_at: datetime_to_timestamp_opt(r.updated_at),
            id: uuid_to_string(r.id),
            is_enabled: r.is_enabled,
            is_revoked: r.is_revoked,
            r#type: r.type_ as i32,
            name: r.name,
            service_id: uuid_opt_to_string_opt(r.service_id),
            user_id: uuid_opt_to_string_opt(r.user_id),
        }
    }
}

impl From<pb::KeyCreateRequest> for KeyCreate {
    fn from(r: pb::KeyCreateRequest) -> Self {
        Self {
            is_enabled: r.is_enabled.unwrap_or(true),
            is_revoked: false,
            type_: KeyType::from_i32(r.r#type),
            name: r.name,
            value: "".to_owned(),
            service_id: string_opt_to_uuid_opt(r.service_id),
            user_id: string_opt_to_uuid_opt(r.user_id),
        }
    }
}

impl From<KeyWithValue> for pb::KeyWithValue {
    fn from(r: KeyWithValue) -> Self {
        let value = r.value.clone();
        Self {
            key: Some(r.into()),
            value,
        }
    }
}

impl From<pb::KeyReadRequest> for KeyRead {
    fn from(r: pb::KeyReadRequest) -> Self {
        Self::IdUser(string_to_uuid(r.id), string_opt_to_uuid_opt(r.user_id))
    }
}

impl From<pb::KeyUpdateRequest> for KeyUpdate {
    fn from(r: pb::KeyUpdateRequest) -> Self {
        Self {
            id: string_to_uuid(r.id),
            is_enabled: r.is_enabled,
            is_revoked: None,
            name: r.name,
        }
    }
}

impl From<pb::ServiceListRequest> for ServiceList {
    fn from(r: pb::ServiceListRequest) -> Self {
        let limit = r.limit.unwrap_or(DEFAULT_LIMIT);
        let gt = string_opt_to_uuid_opt(r.gt);
        let lt = string_opt_to_uuid_opt(r.lt);
        let query = match (gt, lt) {
            (Some(gt), Some(_lt)) => ServiceListQuery::IdGt(gt),
            (Some(gt), None) => ServiceListQuery::IdGt(gt),
            (None, Some(lt)) => ServiceListQuery::IdLt(lt),
            (None, None) => ServiceListQuery::Limit,
        };
        let filter = ServiceListFilter {
            id: string_vec_to_uuid_vec_opt(r.id),
            is_enabled: r.is_enabled,
            limit,
        };
        Self { query, filter }
    }
}

impl From<pb::ServiceCreateRequest> for ServiceCreate {
    fn from(r: pb::ServiceCreateRequest) -> Self {
        Self {
            is_enabled: r.is_enabled.unwrap_or(true),
            name: r.name,
            url: r.url,
            user_allow_register: r.user_allow_register.unwrap_or(false),
            user_email_text: r.user_email_text.unwrap_or("".to_owned()),
            provider_local_url: r.provider_local_url,
            provider_github_oauth2_url: r.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: r.provider_microsoft_oauth2_url,
        }
    }
}

impl From<pb::ServiceReadRequest> for ServiceRead {
    fn from(r: pb::ServiceReadRequest) -> Self {
        Self {
            id: string_to_uuid(r.id),
        }
    }
}

impl From<pb::ServiceUpdateRequest> for ServiceUpdate {
    fn from(r: pb::ServiceUpdateRequest) -> Self {
        Self {
            id: string_to_uuid(r.id),
            is_enabled: r.is_enabled,
            name: r.name,
            url: r.url,
            user_allow_register: r.user_allow_register,
            user_email_text: r.user_email_text,
            provider_local_url: r.provider_local_url,
            provider_github_oauth2_url: r.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: r.provider_microsoft_oauth2_url,
        }
    }
}

impl From<ServiceList> for pb::ServiceListRequest {
    fn from(l: ServiceList) -> Self {
        let id = uuid_vec_opt_to_string_vec(l.filter.id);
        let is_enabled = l.filter.is_enabled;
        let limit = l.filter.limit;
        match l.query {
            ServiceListQuery::Limit => Self {
                gt: None,
                lt: None,
                limit: Some(limit),
                id,
                is_enabled,
            },
            ServiceListQuery::IdGt(gt) => Self {
                gt: Some(uuid_to_string(gt)),
                lt: None,
                limit: Some(limit),
                id,
                is_enabled,
            },
            ServiceListQuery::IdLt(lt) => Self {
                gt: None,
                lt: Some(uuid_to_string(lt)),
                limit: Some(limit),
                id,
                is_enabled,
            },
        }
    }
}

impl From<Service> for pb::Service {
    fn from(r: Service) -> Self {
        Self {
            created_at: datetime_to_timestamp_opt(r.created_at),
            updated_at: datetime_to_timestamp_opt(r.updated_at),
            id: uuid_to_string(r.id),
            is_enabled: r.is_enabled,
            name: r.name,
            url: r.url,
            user_allow_register: r.user_allow_register,
            user_email_text: r.user_email_text,
            provider_local_url: r.provider_local_url,
            provider_github_oauth2_url: r.provider_github_oauth2_url,
            provider_microsoft_oauth2_url: r.provider_microsoft_oauth2_url,
        }
    }
}

impl From<UserTokenAccess> for pb::AuthToken {
    fn from(r: UserTokenAccess) -> Self {
        Self {
            token: r.access_token,
            token_expires: r.access_token_expires,
        }
    }
}

impl From<pb::UserListRequest> for UserList {
    fn from(r: pb::UserListRequest) -> Self {
        let limit = r.limit.unwrap_or(DEFAULT_LIMIT);
        let gt = string_opt_to_uuid_opt(r.gt);
        let lt = string_opt_to_uuid_opt(r.lt);
        let offset_id = string_opt_to_uuid_opt(r.offset_id);
        let query = match (gt, lt, r.name_ge, r.name_le) {
            (Some(gt), _, _, _) => UserListQuery::IdGt(gt),
            (_, Some(lt), _, _) => UserListQuery::IdLt(lt),
            (_, _, Some(name_ge), _) => UserListQuery::NameGe(name_ge, offset_id),
            (_, _, _, Some(name_le)) => UserListQuery::NameLe(name_le, offset_id),
            (_, _, _, _) => UserListQuery::IdGt(Uuid::nil()),
        };
        let filter = UserListFilter {
            id: string_vec_to_uuid_vec_opt(r.id),
            email: string_vec_to_string_vec_opt(r.email),
            limit,
        };
        Self { query, filter }
    }
}

impl From<pb::UserCreateRequest> for UserCreate {
    fn from(r: pb::UserCreateRequest) -> Self {
        let mut create = UserCreate::new(r.is_enabled.unwrap_or(true), r.name, r.email);
        if let Some(locale) = r.locale {
            create = create.locale(locale);
        }
        if let Some(timezone) = r.timezone {
            create = create.timezone(timezone);
        }
        if let Some(password) = r.password {
            create = create
                .with_password(
                    r.password_allow_reset.unwrap_or(false),
                    r.password_require_update.unwrap_or(false),
                    password,
                )
                .unwrap();
        }
        create
    }
}

impl From<pb::UserReadRequest> for UserRead {
    fn from(r: pb::UserReadRequest) -> Self {
        Self::Id(string_to_uuid(r.id))
    }
}

impl From<pb::UserUpdateRequest> for UserUpdate {
    fn from(r: pb::UserUpdateRequest) -> Self {
        Self::new(
            string_to_uuid(r.id),
            r.is_enabled,
            r.name,
            r.locale,
            r.timezone,
            r.password_allow_reset,
            r.password_require_update,
        )
    }
}

impl From<UserList> for pb::UserListRequest {
    fn from(l: UserList) -> Self {
        let id = uuid_vec_opt_to_string_vec(l.filter.id);
        let email = l.filter.email.unwrap_or(Vec::new());
        match l.query {
            UserListQuery::IdGt(gt) => Self {
                gt: Some(uuid_to_string(gt)),
                lt: None,
                name_ge: None,
                name_le: None,
                limit: Some(l.filter.limit),
                offset_id: None,
                id,
                email,
            },
            UserListQuery::IdLt(lt) => Self {
                gt: None,
                lt: Some(uuid_to_string(lt)),
                name_ge: None,
                name_le: None,
                limit: Some(l.filter.limit),
                offset_id: None,
                id,
                email,
            },
            UserListQuery::NameGe(name_ge, offset_id) => Self {
                gt: None,
                lt: None,
                name_ge: Some(name_ge),
                name_le: None,
                limit: Some(l.filter.limit),
                offset_id: uuid_opt_to_string_opt(offset_id),
                id,
                email,
            },
            UserListQuery::NameLe(name_le, offset_id) => Self {
                gt: None,
                lt: None,
                name_ge: None,
                name_le: Some(name_le),
                limit: Some(l.filter.limit),
                offset_id: uuid_opt_to_string_opt(offset_id),
                id,
                email,
            },
        }
    }
}

impl From<User> for pb::User {
    fn from(r: User) -> Self {
        Self {
            created_at: datetime_to_timestamp_opt(r.created_at),
            updated_at: datetime_to_timestamp_opt(r.updated_at),
            id: uuid_to_string(r.id),
            is_enabled: r.is_enabled,
            name: r.name,
            email: r.email,
            locale: r.locale,
            timezone: r.timezone,
            password_allow_reset: r.password_allow_reset,
            password_require_update: r.password_require_update,
        }
    }
}

impl From<pb::User> for User {
    fn from(r: pb::User) -> Self {
        Self {
            created_at: timestamp_opt_to_datetime(r.created_at),
            updated_at: timestamp_opt_to_datetime(r.updated_at),
            id: string_to_uuid(r.id),
            is_enabled: r.is_enabled,
            name: r.name,
            email: r.email,
            locale: r.locale,
            timezone: r.timezone,
            password_allow_reset: r.password_allow_reset,
            password_require_update: r.password_require_update,
            password_hash: None,
        }
    }
}

impl From<UserPasswordMeta> for pb::AuthPasswordMeta {
    fn from(r: UserPasswordMeta) -> Self {
        Self {
            password_strength: r.password_strength.map(|x| x as u32),
            password_pwned: r.password_pwned,
        }
    }
}

impl From<Csrf> for pb::Csrf {
    fn from(r: Csrf) -> Self {
        Self {
            created_at: datetime_to_timestamp_opt(r.created_at),
            key: r.key,
            value: r.value,
            ttl: datetime_to_timestamp_opt(r.ttl),
            service_id: Some(uuid_to_string(r.service_id)),
        }
    }
}

impl UserToken {
    pub fn access_token(&self) -> pb::AuthToken {
        pb::AuthToken {
            token: self.access_token.clone(),
            token_expires: self.access_token_expires,
        }
    }

    pub fn refresh_token(&self) -> pb::AuthToken {
        pb::AuthToken {
            token: self.refresh_token.clone(),
            token_expires: self.refresh_token_expires,
        }
    }
}

impl From<pb::AuditReadRequest> for AuditRead {
    fn from(r: pb::AuditReadRequest) -> Self {
        Self::new(string_to_uuid(r.id)).subject(r.subject)
    }
}

impl From<pb::AuditUpdateRequest> for AuditUpdate {
    fn from(r: pb::AuditUpdateRequest) -> Self {
        Self {
            id: string_to_uuid(r.id),
            status_code: r.status_code.map(|x| x as u16),
            subject: r.subject,
            data: struct_opt_to_value_opt(r.data),
        }
    }
}

impl From<AuditList> for pb::AuditListRequest {
    fn from(l: AuditList) -> Self {
        let id = uuid_vec_opt_to_string_vec(l.filter.id);
        let type_ = l.filter.type_.unwrap_or(Vec::new());
        let subject = l.filter.subject.unwrap_or(Vec::new());
        let service_id = uuid_vec_opt_to_string_vec(l.filter.service_id);
        let user_id = uuid_vec_opt_to_string_vec(l.filter.user_id);
        match l.query {
            AuditListQuery::CreatedLe(le, limit, offset_id) => Self {
                ge: None,
                le: datetime_to_timestamp_opt(le),
                limit: Some(limit),
                offset_id: uuid_opt_to_string_opt(offset_id),
                id,
                r#type: type_,
                subject,
                service_id,
                user_id,
            },
            AuditListQuery::CreatedGe(ge, limit, offset_id) => Self {
                ge: datetime_to_timestamp_opt(ge),
                le: None,
                limit: Some(limit),
                offset_id: uuid_opt_to_string_opt(offset_id),
                id,
                r#type: type_,
                subject,
                service_id,
                user_id,
            },
            AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id) => Self {
                ge: datetime_to_timestamp_opt(ge),
                le: datetime_to_timestamp_opt(le),
                limit: Some(limit),
                offset_id: uuid_opt_to_string_opt(offset_id),
                id,
                r#type: type_,
                subject,
                service_id,
                user_id,
            },
        }
    }
}

impl From<Audit> for pb::Audit {
    fn from(r: Audit) -> Self {
        Self {
            created_at: datetime_to_timestamp_opt(r.created_at),
            updated_at: datetime_to_timestamp_opt(r.updated_at),
            id: uuid_to_string(r.id),
            user_agent: r.user_agent,
            remote: r.remote,
            forwarded: r.forwarded,
            status_code: r.status_code.map(|x| x as u32),
            r#type: r.type_,
            subject: r.subject,
            data: value_to_struct_opt(r.data),
            key_id: uuid_opt_to_string_opt(r.key_id),
            service_id: uuid_opt_to_string_opt(r.service_id),
            user_id: uuid_opt_to_string_opt(r.user_id),
            user_key_id: uuid_opt_to_string_opt(r.user_key_id),
        }
    }
}

impl pb::ServiceCreateRequest {
    pub fn new<N, U>(is_enabled: bool, name: N, url: U) -> Self
    where
        N: Into<String>,
        U: Into<String>,
    {
        Self {
            name: name.into(),
            url: url.into(),
            is_enabled: Some(is_enabled),
            user_allow_register: None,
            user_email_text: None,
            provider_local_url: None,
            provider_github_oauth2_url: None,
            provider_microsoft_oauth2_url: None,
        }
    }

    pub fn user_allow_register(mut self, user_allow_register: bool) -> Self {
        self.user_allow_register = Some(user_allow_register);
        self
    }

    pub fn user_email_text<S: Into<String>>(mut self, user_email_text: S) -> Self {
        self.user_email_text = Some(user_email_text.into());
        self
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

impl pb::KeyCreateRequest {
    pub fn new<N>(is_enabled: bool, type_: KeyType, name: N) -> Self
    where
        N: Into<String>,
    {
        Self {
            r#type: type_ as i32,
            name: name.into(),
            is_enabled: Some(is_enabled),
            service_id: None,
            user_id: None,
        }
    }

    pub fn with_service_id<N>(is_enabled: bool, type_: KeyType, name: N, service_id: String) -> Self
    where
        N: Into<String>,
    {
        Self {
            r#type: type_ as i32,
            name: name.into(),
            is_enabled: Some(is_enabled),
            service_id: Some(service_id),
            user_id: None,
        }
    }

    pub fn with_user_id<N>(is_enabled: bool, type_: KeyType, name: N, user_id: String) -> Self
    where
        N: Into<String>,
    {
        Self {
            r#type: type_ as i32,
            name: name.into(),
            is_enabled: Some(is_enabled),
            service_id: None,
            user_id: Some(user_id),
        }
    }
}

impl pb::KeyListRequest {
    pub fn limit(limit: i64) -> Self {
        Self {
            gt: None,
            lt: None,
            limit: Some(limit),
            id: Vec::new(),
            is_enabled: None,
            is_revoked: None,
            r#type: Vec::new(),
            service_id: Vec::new(),
            user_id: Vec::new(),
        }
    }

    pub fn limit_id(limit: i64, id: Vec<String>) -> Self {
        let mut x = Self::limit(limit);
        x.id = id;
        x
    }

    pub fn gt_limit_id(gt: String, limit: i64, id: Vec<String>) -> Self {
        let mut x = Self::limit_id(limit, id);
        x.gt = Some(gt);
        x
    }

    pub fn lt_limit_id(lt: String, limit: i64, id: Vec<String>) -> Self {
        let mut x = Self::limit_id(limit, id);
        x.lt = Some(lt);
        x
    }
}

impl pb::UserCreateRequest {
    pub fn new<N, E>(is_enabled: bool, name: N, email: E) -> Self
    where
        N: Into<String>,
        E: Into<String>,
    {
        Self {
            name: name.into(),
            email: email.into(),
            is_enabled: Some(is_enabled),
            locale: None,
            timezone: None,
            password_allow_reset: None,
            password_require_update: None,
            password: None,
        }
    }

    pub fn locale<L>(mut self, locale: L) -> Self
    where
        L: Into<String>,
    {
        self.locale = Some(locale.into());
        self
    }

    pub fn timezone<T>(mut self, timezone: T) -> Self
    where
        T: Into<String>,
    {
        self.timezone = Some(timezone.into());
        self
    }

    pub fn with_password<P>(
        mut self,
        password_allow_reset: bool,
        password_require_update: bool,
        password: P,
    ) -> Self
    where
        P: Into<String>,
    {
        self.password_allow_reset = Some(password_allow_reset);
        self.password_require_update = Some(password_require_update);
        self.password = Some(password.into());
        self
    }
}

impl pb::AuthKeyRequest {
    pub fn new<K>(key: K, audit: Option<String>) -> Self
    where
        K: Into<String>,
    {
        Self {
            key: key.into(),
            audit,
        }
    }
}

impl pb::AuthTokenRequest {
    pub fn new<T>(token: T, audit: Option<String>) -> Self
    where
        T: Into<String>,
    {
        Self {
            token: token.into(),
            audit,
        }
    }
}

impl pb::AuditCreateRequest {
    pub fn new(type_: String) -> Self {
        Self {
            r#type: type_,
            subject: None,
            data: None,
            user_id: None,
            user_key_id: None,
        }
    }
}

impl pb::ServiceListRequest {
    pub fn limit_id(limit: i64, id: Vec<String>) -> Self {
        Self {
            gt: None,
            lt: None,
            limit: Some(limit),
            id: id,
            is_enabled: None,
        }
    }

    pub fn gt_limit_id(gt: String, limit: i64, id: Vec<String>) -> Self {
        let mut x = Self::limit_id(limit, id);
        x.gt = Some(gt);
        x
    }

    pub fn lt_limit_id(lt: String, limit: i64, id: Vec<String>) -> Self {
        let mut x = Self::limit_id(limit, id);
        x.lt = Some(lt);
        x
    }
}

impl pb::AuditListRequest {
    pub fn ge_limit(ge: Option<prost_types::Timestamp>, limit: i64) -> Self {
        Self {
            ge,
            le: None,
            limit: Some(limit),
            offset_id: None,
            id: Vec::new(),
            r#type: Vec::new(),
            subject: Vec::new(),
            service_id: Vec::new(),
            user_id: Vec::new(),
        }
    }

    pub fn ge_offset_limit(
        ge: Option<prost_types::Timestamp>,
        offset_id: String,
        limit: i64,
    ) -> Self {
        Self {
            ge,
            le: None,
            limit: Some(limit),
            offset_id: Some(offset_id),
            id: Vec::new(),
            r#type: Vec::new(),
            subject: Vec::new(),
            service_id: Vec::new(),
            user_id: Vec::new(),
        }
    }

    pub fn le_offset_limit(
        le: Option<prost_types::Timestamp>,
        offset_id: String,
        limit: i64,
    ) -> Self {
        Self {
            ge: None,
            le,
            limit: Some(limit),
            offset_id: Some(offset_id),
            id: Vec::new(),
            r#type: Vec::new(),
            subject: Vec::new(),
            service_id: Vec::new(),
            user_id: Vec::new(),
        }
    }

    pub fn ge_le_offset_limit(
        ge: Option<prost_types::Timestamp>,
        le: Option<prost_types::Timestamp>,
        offset_id: String,
        limit: i64,
    ) -> Self {
        Self {
            ge,
            le,
            limit: Some(limit),
            offset_id: Some(offset_id),
            id: Vec::new(),
            r#type: Vec::new(),
            subject: Vec::new(),
            service_id: Vec::new(),
            user_id: Vec::new(),
        }
    }

    pub fn type_subject(type_: Vec<String>, subject: Vec<String>) -> Self {
        Self {
            ge: None,
            le: None,
            limit: None,
            offset_id: None,
            id: Vec::new(),
            r#type: type_,
            subject: subject,
            service_id: Vec::new(),
            user_id: Vec::new(),
        }
    }
}

impl pb::UserUpdateRequest {
    pub fn new(id: String) -> Self {
        Self {
            id,
            is_enabled: None,
            name: None,
            locale: None,
            timezone: None,
            password_allow_reset: None,
            password_require_update: None,
        }
    }

    pub fn is_enabled(mut self, is_enabled: bool) -> Self {
        self.is_enabled = Some(is_enabled);
        self
    }

    pub fn name<N>(mut self, name: N) -> Self
    where
        N: Into<String>,
    {
        self.name = Some(name.into());
        self
    }
}

impl pb::UserListRequest {
    pub fn limit(limit: i64) -> Self {
        let mut x = Self::default();
        x.limit = Some(limit);
        x
    }

    pub fn id(id: Vec<String>) -> Self {
        let mut x = Self::default();
        x.id = id;
        x
    }

    pub fn email(email: Vec<String>) -> Self {
        let mut x = Self::default();
        x.email = email;
        x
    }

    pub fn gt_limit(gt: String, limit: i64) -> Self {
        let mut x = Self::limit(limit);
        x.gt = Some(gt);
        x
    }

    pub fn lt_limit(lt: String, limit: i64) -> Self {
        let mut x = Self::limit(limit);
        x.lt = Some(lt);
        x
    }

    pub fn name_ge_limit_id(name_ge: String, limit: i64, id: Vec<String>) -> Self {
        let mut x = Self::limit(limit);
        x.name_ge = Some(name_ge);
        x.id = id;
        x
    }

    pub fn name_ge_offset_limit_id(
        name_ge: String,
        offset_id: String,
        limit: i64,
        id: Vec<String>,
    ) -> Self {
        let mut x = Self::name_ge_limit_id(name_ge, limit, id);
        x.offset_id = Some(offset_id);
        x
    }

    pub fn name_le_offset_limit_id(
        name_le: String,
        offset_id: String,
        limit: i64,
        id: Vec<String>,
    ) -> Self {
        let mut x = Self::limit(limit);
        x.name_le = Some(name_le);
        x.offset_id = Some(offset_id);
        x.id = id;
        x
    }
}

impl pb::AuthLoginRequest {
    pub fn new<E, P>(email: E, password: P) -> Self
    where
        E: Into<String>,
        P: Into<String>,
    {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

impl pb::AuthResetPasswordRequest {
    pub fn new<E>(email: E) -> Self
    where
        E: Into<String>,
    {
        Self {
            email: email.into(),
        }
    }
}

impl pb::AuthCsrfCreateRequest {
    pub fn new(expires_s: i64) -> Self {
        Self {
            expires_s: Some(expires_s),
        }
    }
}

impl pb::AuthCsrfVerifyRequest {
    pub fn new<C>(csrf: C) -> Self
    where
        C: Into<String>,
    {
        Self {
            csrf: csrf.into(),
            audit: None,
        }
    }
}

impl pb::AuthTotpRequest {
    pub fn new<U, T>(user_id: U, totp: T) -> Self
    where
        U: Into<String>,
        T: Into<String>,
    {
        Self {
            user_id: user_id.into(),
            totp: totp.into(),
        }
    }
}

impl pb::AuthRegisterRequest {
    pub fn new<N, E>(name: N, email: E) -> Self
    where
        N: Into<String>,
        E: Into<String>,
    {
        let mut x = Self::default();
        x.name = name.into();
        x.email = email.into();
        x
    }
}

impl pb::AuthResetPasswordConfirmRequest {
    pub fn new<T, P>(token: T, password: P) -> Self
    where
        T: Into<String>,
        P: Into<String>,
    {
        let mut x = Self::default();
        x.token = token.into();
        x.password = password.into();
        x
    }
}
