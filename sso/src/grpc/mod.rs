//! gRPC server and clients.
mod client;
mod http;
mod options;
pub mod pb {
    //! Generated protobuf server and client items.
    tonic::include_proto!("sso");
}

pub use crate::grpc::pb::sso_client::SsoClient as Client;
pub use crate::grpc::{client::*, http::*, options::*};

use crate::{
    api::{self, ApiError, ValidateRequest},
    *,
};
use chrono::{DateTime, Utc};
use core::pin::Pin;
use lettre::{file::FileTransport, SmtpClient, Transport};
use lettre_email::Email;
use prometheus::{HistogramTimer, HistogramVec, IntCounterVec};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tonic::{metadata::MetadataMap, Request, Response, Status};
use uuid::Uuid;

/// gRPC server.
#[derive(Clone)]
pub struct Server {
    options: ServerOptions,
    driver: Arc<Box<dyn Driver>>,
    client: Arc<reqwest::Client>,
    smtp_client: Arc<Option<SmtpClient>>,
    count: IntCounterVec,
    latency: HistogramVec,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server {{ driver }}")
    }
}

impl Server {
    /// Returns a new `Server`.
    pub fn new(driver: Box<dyn Driver>, options: ServerOptions) -> Self {
        let client = options.client().unwrap();
        let smtp_client = options.smtp_client().unwrap();
        let (count, latency) = Metrics::http_metrics();
        Self {
            options,
            driver: Arc::new(driver),
            client: Arc::new(client),
            smtp_client: Arc::new(smtp_client),
            count,
            latency,
        }
    }

    /// Returns reference to driver.
    pub fn driver(&self) -> Arc<Box<dyn Driver>> {
        self.driver.clone()
    }

    /// Build email callback function. Must be called from blocking context.
    /// If client is None and file directory path is provided, file transport is used.
    pub fn smtp_email(&self) -> Box<dyn FnOnce(TemplateEmail) -> DriverResult<()>> {
        let client = self.smtp_client.clone();
        let from_email = self.options.smtp_from_email();
        let smtp_file = self.options.smtp_file();

        Box::new(move |email| {
            let email_builder = Email::builder()
                .to((email.to_email, email.to_name))
                .subject(email.subject)
                .text(email.text);

            match (client.as_ref(), smtp_file) {
                (Some(client), _) => {
                    let email = email_builder
                        .from((from_email.unwrap(), email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let mut transport = client.clone().transport();
                    transport.send(email.into()).map_err(DriverError::Lettre)?;
                    Ok(())
                }
                (_, Some(smtp_file)) => {
                    let email = email_builder
                        .from(("file@localhost", email.from_name))
                        .build()
                        .map_err(DriverError::LettreEmail)?;

                    let path = PathBuf::from(smtp_file);
                    let mut transport = FileTransport::new(path);
                    transport
                        .send(email.into())
                        .map_err(DriverError::LettreFile)?;
                    Ok(())
                }
                (None, None) => Err(DriverError::SmtpDisabled),
            }
        })
    }

    pub fn metrics_start(&self, path: &str) -> (HistogramTimer, IntCounterVec) {
        let timer = self.latency.with_label_values(&[path]).start_timer();
        (timer, self.count.clone())
    }

    pub fn metrics_end(
        &self,
        timer: HistogramTimer,
        count: IntCounterVec,
        path: &str,
        status: &str,
    ) {
        timer.observe_duration();
        count.with_label_values(&[path, status]).inc_by(1);
    }
}

#[tonic::async_trait]
impl pb::sso_server::Sso for Server {
    async fn ping(&self, _: Request<()>) -> Result<Response<String>, Status> {
        Err(Status::not_found(""))
    }

    async fn metrics(&self, _: Request<()>) -> Result<Response<String>, Status> {
        Err(Status::not_found(""))
    }

    async fn audit_list(
        &self,
        request: Request<pb::AuditListRequest>,
    ) -> Result<Response<pb::AuditListReply>, Status> {
        let audit_meta = request_audit_meta(request.remote_addr(), request.metadata())?;
        let auth = request_authorisation(request.metadata())?;
        let req: AuditList = request.into_inner().try_into()?;
        AuditList::status_validate(&req)?;

        let driver = self.driver.clone();
        let reply = blocking::<_, Status, _>(move || {
            let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditList);
            let res: Result<Vec<Audit>, Status> = {
                let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

                driver
                    .as_ref()
                    .audit_list(&req, service.map(|s| s.id))
                    .map_err(ApiError::BadRequest)
                    .map_err::<Status, _>(Into::into)
            };
            let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
            let reply = pb::AuditListReply {
                meta: Some(req.into()),
                data: data.into_iter().map::<pb::Audit, _>(|x| x.into()).collect(),
            };
            Ok(reply)
        })
        .await?;
        Ok(Response::new(reply))
    }

    async fn audit_create(
        &self,
        request: Request<pb::AuditCreateRequest>,
    ) -> Result<Response<pb::AuditReadReply>, Status> {
        let audit_meta = request_audit_meta(request.remote_addr(), request.metadata())?;
        let auth = request_authorisation(request.metadata())?;
        let req = request.into_inner();
        let data = serde_json::to_value(req.data).unwrap();
        let req = AuditCreate::new(audit_meta.clone(), req.r#type)
            .subject(req.subject)
            .data(Some(data))
            .user_id(string_opt_to_uuid_opt(req.user_id))
            .user_key_id(string_opt_to_uuid_opt(req.user_key_id));
        AuditCreate::status_validate(&req)?;

        let driver = self.driver.clone();
        let reply = blocking::<_, Status, _>(move || {
            let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditCreate);
            let res: Result<Audit, Status> = {
                let _service =
                    pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                        .map_err(ApiError::Unauthorised)?;

                audit
                    .create2(driver.as_ref().as_ref(), req)
                    .map_err(ApiError::BadRequest)
                    .map_err::<Status, _>(Into::into)
            };
            let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
            let reply = pb::AuditReadReply {
                data: Some(data.into()),
            };
            Ok(reply)
        })
        .await?;
        Ok(Response::new(reply))
    }

    async fn audit_read(
        &self,
        request: Request<pb::AuditReadRequest>,
    ) -> Result<Response<pb::AuditReadReply>, Status> {
        let audit_meta = request_audit_meta(request.remote_addr(), request.metadata())?;
        let auth = request_authorisation(request.metadata())?;
        let req: AuditRead = request.into_inner().try_into()?;
        AuditRead::status_validate(&req)?;

        let driver = self.driver.clone();
        let reply = blocking::<_, Status, _>(move || {
            let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditRead);
            let res: Result<Audit, Status> = {
                let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

                driver
                    .audit_read(&req, service.map(|x| x.id))
                    .map_err(ApiError::BadRequest)
                    .map_err::<Status, _>(Into::into)?
                    .ok_or_else(|| {
                        let e: Status = ApiError::NotFound(DriverError::AuditNotFound).into();
                        e
                    })
            };
            let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
            let reply = pb::AuditReadReply {
                data: Some(data.into()),
            };
            Ok(reply)
        })
        .await?;
        Ok(Response::new(reply))
    }

    async fn audit_update(
        &self,
        request: Request<pb::AuditUpdateRequest>,
    ) -> Result<Response<pb::AuditReadReply>, Status> {
        let audit_meta = request_audit_meta(request.remote_addr(), request.metadata())?;
        let auth = request_authorisation(request.metadata())?;
        let req: AuditUpdate = request.into_inner().try_into()?;
        AuditUpdate::status_validate(&req)?;

        let driver = self.driver.clone();
        let reply = blocking::<_, Status, _>(move || {
            let mut audit = AuditBuilder::new(audit_meta, AuditType::AuditUpdate);
            let res: Result<Audit, Status> = {
                let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                    .map_err(ApiError::Unauthorised)?;

                driver
                    .audit_update(&req, service.map(|x| x.id))
                    .map_err(ApiError::BadRequest)
                    .map_err::<Status, _>(Into::into)
            };
            let data = api::result_audit_err(driver.as_ref().as_ref(), &audit, res)?;
            let reply = pb::AuditReadReply {
                data: Some(data.into()),
            };
            Ok(reply)
        })
        .await?;
        Ok(Response::new(reply))
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
