use crate::{grpc::pb, *};
use chrono::{DateTime, Utc};
use core::pin::Pin;
use std::convert::TryFrom;
use std::future::Future;
use std::net::SocketAddr;
use tonic::{metadata::MetadataMap, Status};
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

/// Get audit meta and authorisation header from request metadata.
pub fn request_audit_auth(
    remote: Option<SocketAddr>,
    metadata: &MetadataMap,
) -> Result<(AuditMeta, Option<String>), Status> {
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

pub fn datetime_to_timestamp_opt(dt: DateTime<Utc>) -> Option<prost_types::Timestamp> {
    let st: std::time::SystemTime = dt.into();
    let ti: prost_types::Timestamp = st.into();
    Some(ti)
}

pub fn string_to_uuid(s: String) -> Uuid {
    serde_json::from_str(&s).unwrap()
}

pub fn string_opt_to_uuid_opt(s: Option<String>) -> Option<Uuid> {
    match s {
        Some(s) => {
            let u: Uuid = serde_json::from_str(&s).unwrap();
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
                .map(|s| serde_json::from_str::<Uuid>(&s).unwrap())
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

impl TryFrom<pb::KeyListRequest> for KeyList {
    type Error = Status;

    fn try_from(r: pb::KeyListRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl From<KeyList> for pb::KeyListRequest {
    fn from(l: KeyList) -> Self {
        unimplemented!();
    }
}

impl From<Key> for pb::Key {
    fn from(r: Key) -> Self {
        unimplemented!();
    }
}

impl From<KeyWithValue> for pb::Key {
    fn from(r: KeyWithValue) -> Self {
        unimplemented!();
    }
}

impl TryFrom<pb::KeyCreateRequest> for KeyCreate {
    type Error = Status;

    fn try_from(r: pb::KeyCreateRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl From<KeyWithValue> for pb::KeyWithValue {
    fn from(r: KeyWithValue) -> Self {
        unimplemented!();
    }
}

impl TryFrom<pb::KeyReadRequest> for KeyRead {
    type Error = Status;

    fn try_from(r: pb::KeyReadRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::KeyUpdateRequest> for KeyUpdate {
    type Error = Status;

    fn try_from(r: pb::KeyUpdateRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::ServiceListRequest> for ServiceList {
    type Error = Status;

    fn try_from(r: pb::ServiceListRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::ServiceCreateRequest> for ServiceCreate {
    type Error = Status;

    fn try_from(r: pb::ServiceCreateRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::ServiceReadRequest> for ServiceRead {
    type Error = Status;

    fn try_from(r: pb::ServiceReadRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::ServiceUpdateRequest> for ServiceUpdate {
    type Error = Status;

    fn try_from(r: pb::ServiceUpdateRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl From<ServiceList> for pb::ServiceListRequest {
    fn from(l: ServiceList) -> Self {
        unimplemented!();
    }
}

impl From<Service> for pb::Service {
    fn from(r: Service) -> Self {
        unimplemented!();
    }
}

impl From<UserTokenAccess> for pb::AuthToken {
    fn from(r: UserTokenAccess) -> Self {
        unimplemented!();
    }
}

impl TryFrom<pb::UserListRequest> for UserList {
    type Error = Status;

    fn try_from(r: pb::UserListRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::UserCreateRequest> for UserCreate {
    type Error = Status;

    fn try_from(r: pb::UserCreateRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::UserReadRequest> for UserRead {
    type Error = Status;

    fn try_from(r: pb::UserReadRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl TryFrom<pb::UserUpdateRequest> for UserUpdate {
    type Error = Status;

    fn try_from(r: pb::UserUpdateRequest) -> Result<Self, Self::Error> {
        unimplemented!();
    }
}

impl From<UserList> for pb::UserListRequest {
    fn from(l: UserList) -> Self {
        unimplemented!();
    }
}

impl From<User> for pb::User {
    fn from(r: User) -> Self {
        unimplemented!();
    }
}

impl From<UserPasswordMeta> for pb::AuthPasswordMeta {
    fn from(r: UserPasswordMeta) -> Self {
        unimplemented!();
    }
}

impl From<Csrf> for pb::Csrf {
    fn from(r: Csrf) -> Self {
        unimplemented!();
    }
}

impl UserToken {
    pub fn access_token(&self) -> pb::AuthToken {
        unimplemented!();
    }

    pub fn refresh_token(&self) -> pb::AuthToken {
        unimplemented!();
    }
}

impl TryFrom<pb::AuditListRequest> for AuditList {
    type Error = Status;

    fn try_from(r: pb::AuditListRequest) -> Result<Self, Self::Error> {
        let limit = r.limit.unwrap_or(DEFAULT_LIMIT);
        let ge = timestamp_opt_to_datetime_opt(r.ge);
        let le = timestamp_opt_to_datetime_opt(r.le);
        let offset_id = string_opt_to_uuid_opt(r.offset_id);
        let query = match (ge, le) {
            (Some(ge), Some(le)) => AuditListQuery::CreatedLeAndGe(le, ge, limit, offset_id),
            (Some(ge), None) => AuditListQuery::CreatedGe(ge, limit, offset_id),
            (None, Some(le)) => AuditListQuery::CreatedLe(le, limit, offset_id),
            (None, None) => AuditListQuery::CreatedLe(Utc::now(), limit, offset_id),
        };
        let filter = AuditListFilter {
            id: string_vec_to_uuid_vec_opt(r.id),
            type_: string_vec_to_string_vec_opt(r.r#type),
            subject: string_vec_to_string_vec_opt(r.subject),
            service_id: string_vec_to_uuid_vec_opt(r.service_id),
            user_id: string_vec_to_uuid_vec_opt(r.user_id),
        };
        Ok(AuditList { query, filter })
    }
}

impl TryFrom<pb::AuditReadRequest> for AuditRead {
    type Error = Status;

    fn try_from(r: pb::AuditReadRequest) -> Result<Self, Self::Error> {
        Ok(Self::new(string_to_uuid(r.id)).subject(r.subject))
    }
}

impl TryFrom<pb::AuditUpdateRequest> for AuditUpdate {
    type Error = Status;

    fn try_from(r: pb::AuditUpdateRequest) -> Result<Self, Self::Error> {
        let data = serde_json::to_value(r.data).unwrap();
        Ok(Self {
            id: string_to_uuid(r.id),
            status_code: r.status_code.map(|x| x as u16),
            subject: r.subject,
            data: Some(data),
        })
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
        let data: std::collections::HashMap<String, String> =
            serde_json::from_value(r.data).unwrap();
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
            data,
            key_id: uuid_opt_to_string_opt(r.key_id),
            service_id: uuid_opt_to_string_opt(r.service_id),
            user_id: uuid_opt_to_string_opt(r.user_id),
            user_key_id: uuid_opt_to_string_opt(r.user_key_id),
        }
    }
}
