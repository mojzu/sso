//! Blocking client.
use crate::pb::{self, sso_client::SsoClient, Empty, Text};
use http::{HeaderValue, Uri};
use std::fmt;
use std::str::FromStr;
use tokio::runtime::{Builder, Runtime};
use tonic::transport::Channel;

impl fmt::Debug for SsoClient<tonic::transport::Channel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SsoClient {{ }}")
    }
}

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = StdError> = ::std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub uri: Uri,
    pub authorisation: String,
}

impl ClientOptions {
    pub fn new<U: AsRef<str>>(uri: U) -> Self {
        Self {
            uri: Uri::from_str(uri.as_ref()).unwrap(),
            authorisation: String::from(""),
        }
    }

    pub fn authorisation<A: Into<String>>(mut self, authorisation: A) -> Self {
        self.authorisation = authorisation.into();
        self
    }
}

// The order of the fields in this struct is important. The runtime must be the first field and the
// client must be the last field so that when `ClientBlocking` is dropped the client is dropped
// before the runtime. Not doing this will result in a deadlock when dropped.
pub struct ClientBlocking {
    rt: Runtime,
    client: SsoClient<tonic::transport::Channel>,
}

impl fmt::Debug for ClientBlocking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientBlocking {{ rt: {:?}, client }}", self.rt)
    }
}

impl ClientBlocking {
    pub fn connect(options: &ClientOptions) -> Result<Self, tonic::transport::Error> {
        let mut rt = Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let authorisation = options.authorisation.to_owned();
        let channel = rt.block_on(
            Channel::builder(options.uri.clone())
                .intercept_headers(move |headers| {
                    headers.insert(
                        "authorization",
                        HeaderValue::from_str(authorisation.as_ref()).unwrap(),
                    );
                })
                .connect(),
        )?;

        let client = SsoClient::new(channel);
        Ok(Self { rt, client })
    }

    pub fn ping(
        &mut self,
        request: impl tonic::IntoRequest<Empty>,
    ) -> Result<tonic::Response<Text>, tonic::Status> {
        self.rt.block_on(self.client.ping(request))
    }

    pub fn metrics(
        &mut self,
        request: impl tonic::IntoRequest<Empty>,
    ) -> Result<tonic::Response<Text>, tonic::Status> {
        self.rt.block_on(self.client.metrics(request))
    }

    pub fn audit_list(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditListRequest>,
    ) -> Result<tonic::Response<pb::AuditListReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_list(request))
    }

    pub fn audit_create(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditCreateRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_create(request))
    }

    pub fn audit_read(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditReadRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_read(request))
    }

    pub fn audit_update(
        &mut self,
        request: impl tonic::IntoRequest<pb::AuditUpdateRequest>,
    ) -> Result<tonic::Response<pb::AuditReadReply>, tonic::Status> {
        self.rt.block_on(self.client.audit_update(request))
    }
}
