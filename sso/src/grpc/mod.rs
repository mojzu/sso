//! gRPC server and clients.
mod client;
pub mod pb {
    //! Generated protobuf server and client items.
    tonic::include_proto!("sso");
}

pub use crate::grpc::client::*;
pub use crate::grpc::pb::sso_client::SsoClient as Client;

// use crate::pb::{AuditListReply, AuditListRequest, Empty, Audit, Text};
use crate::{
    api, pattern, Audit, AuditMeta, Driver, HEADER_AUTHORISATION_NAME,
    HEADER_USER_AUTHORISATION_NAME,
};
use chrono::{DateTime, Utc};
use core::pin::Pin;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{
    body::{Body, BoxBody},
    metadata::MetadataMap,
    Request, Response, Status,
};
use uuid::Uuid;

/// gRPC server.
#[derive(Clone)]
pub struct Server {
    driver: Arc<Box<dyn Driver>>,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server {{ driver }}")
    }
}

impl Server {
    /// Returns a new `Server`.
    pub fn new(driver: Box<dyn Driver>) -> Self {
        Self {
            driver: Arc::new(driver),
        }
    }
}

#[tonic::async_trait]
impl pb::sso_server::Sso for Server {
    async fn ping(&self, request: Request<pb::Empty>) -> Result<Response<pb::Text>, Status> {
        println!("Got a request: {:?}", request);

        let reply = pb::Text {
            text: format!("Hello!"),
        };

        Ok(Response::new(reply))
    }

    async fn metrics(&self, request: Request<pb::Empty>) -> Result<Response<pb::Text>, Status> {
        Ok(Response::new(pb::Text {
            text: "# prometheus".to_owned(),
        }))
    }

    async fn audit_list(
        &self,
        request: Request<pb::AuditListRequest>,
    ) -> Result<Response<pb::AuditListReply>, Status> {
        let driver = self.driver.clone();
        let audit_meta = request_audit_meta(request.remote_addr(), request.metadata())?;
        let auth = request_authorisation(request.metadata())?;
        let req: api::AuditListRequest = request.into_inner().try_into()?;

        let res =
            blocking(move || api::audit_list(driver.as_ref().as_ref(), audit_meta, auth, req))
                .await?;

        Ok(Response::new(res.try_into()?))
    }
}

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

/// Get audit meta from request metadata.
pub fn request_audit_meta(
    remote: Option<SocketAddr>,
    metadata: &MetadataMap,
) -> Result<AuditMeta, Status> {
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
    Ok(AuditMeta::new(user_agent, remote, forwarded, user))
}

// Get authorisation header from request metadata.
pub fn request_authorisation(metadata: &MetadataMap) -> Result<Option<String>, Status> {
    let auth = match metadata.get(HEADER_AUTHORISATION_NAME) {
        Some(value) => Some(value.to_str().unwrap().to_owned()),
        None => None,
    };
    Ok(auth)
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

pub fn datetime_opt_to_timestamp_opt(dt: Option<DateTime<Utc>>) -> Option<prost_types::Timestamp> {
    match dt {
        Some(dt) => {
            let st: std::time::SystemTime = dt.into();
            let ti: prost_types::Timestamp = st.into();
            Some(ti)
        }
        None => None,
    }
}

pub fn string_to_uuid_opt(s: String) -> Option<Uuid> {
    if s.is_empty() {
        None
    } else {
        let u: Uuid = serde_json::from_str(&s).unwrap();
        Some(u)
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

pub fn uuid_opt_to_string(u: Option<Uuid>) -> String {
    match u {
        Some(u) => format!("{}", u),
        None => "".to_owned(),
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

impl TryFrom<pb::AuditListRequest> for api::AuditListRequest {
    type Error = Status;

    fn try_from(r: pb::AuditListRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            ge: timestamp_opt_to_datetime_opt(r.ge),
            le: timestamp_opt_to_datetime_opt(r.le),
            limit: Some(r.limit),
            offset_id: string_to_uuid_opt(r.offset_id),
            id: string_vec_to_uuid_vec_opt(r.id),
            type_: string_vec_to_string_vec_opt(r.r#type),
            subject: string_vec_to_string_vec_opt(r.subject),
            service_id: string_vec_to_uuid_vec_opt(r.service_id),
            user_id: string_vec_to_uuid_vec_opt(r.user_id),
        })
    }
}

impl TryFrom<api::AuditListRequest> for pb::AuditListRequest {
    type Error = Status;

    fn try_from(r: api::AuditListRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            ge: datetime_opt_to_timestamp_opt(r.ge),
            le: datetime_opt_to_timestamp_opt(r.le),
            limit: r.limit.unwrap_or(0),
            offset_id: uuid_opt_to_string(r.offset_id),
            id: uuid_vec_opt_to_string_vec(r.id),
            r#type: r.type_.unwrap_or(Vec::new()),
            subject: r.subject.unwrap_or(Vec::new()),
            service_id: uuid_vec_opt_to_string_vec(r.service_id),
            user_id: uuid_vec_opt_to_string_vec(r.user_id),
        })
    }
}

impl From<Audit> for pb::Audit {
    fn from(r: Audit) -> Self {
        unimplemented!();
    }
}

impl TryFrom<api::AuditListResponse> for pb::AuditListReply {
    type Error = Status;

    fn try_from(r: api::AuditListResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: Some(r.meta.try_into()?),
            data: r
                .data
                .into_iter()
                .map::<pb::Audit, _>(|x| x.into())
                .collect(),
        })
    }
}
