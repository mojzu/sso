//! Blocking client.
use crate::pb::{sso_client::SsoClient, Empty, Text};
use tokio::runtime::{Builder, Runtime};

type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = StdError> = ::std::result::Result<T, E>;

// The order of the fields in this struct is important. The runtime must be the first field and the
// client must be the last field so that when `SsoClientBlocking` is dropped the client is dropped
// before the runtime. Not doing this will result in a deadlock when dropped.
pub struct SsoClientBlocking {
    rt: Runtime,
    client: SsoClient<tonic::transport::Channel>,
}

impl SsoClientBlocking {
    pub fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let mut rt = Builder::new()
            .basic_scheduler()
            .enable_all()
            .build()
            .unwrap();

        let client = rt.block_on(SsoClient::connect(dst))?;

        Ok(Self { rt, client })
    }

    pub fn ping(
        &mut self,
        request: impl tonic::IntoRequest<Empty>,
    ) -> Result<tonic::Response<Text>, tonic::Status> {
        self.rt.block_on(self.client.ping(request))
    }
}
