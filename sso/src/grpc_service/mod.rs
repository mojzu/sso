mod method;
mod server;

pub use crate::grpc_service::server::*;

use crate::grpc::pb::sso_service_client::SsoServiceClient;
use std::fmt;
use tonic::transport::Channel;

/// gRPC service asynchronous client.
pub type GrpcServiceClient = SsoServiceClient<Channel>;

impl fmt::Debug for SsoServiceClient<Channel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GrpcServiceClient {{ }}")
    }
}
