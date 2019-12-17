//! Blocking client.
use crate::pb::{sso_client::SsoClient, Empty, Text};
use http::{HeaderValue, Uri};
use tokio::runtime::{Builder, Runtime};
use tonic::transport::Channel;
use std::str::FromStr;

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = StdError> = ::std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct ClientOptions {
    pub uri: Uri,
    pub authorisation: String,
}

impl ClientOptions {
    pub fn new(uri: &str) -> Self {
        Self {
            uri: Uri::from_str(uri).unwrap(),
            authorisation: String::from(""),
        }
    }

    pub fn authorisation(mut self, authorisation: &str) -> Self {
        self.authorisation = authorisation.to_owned();
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
}
