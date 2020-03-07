mod method;
mod server;

pub use crate::grpc_service::server::*;

use crate::grpc::pb::service_service_client::ServiceServiceClient;
use std::fmt;
use tonic::transport::Channel;

/// gRPC service asynchronous client.
pub type GrpcServiceClient = ServiceServiceClient<Channel>;

impl fmt::Debug for ServiceServiceClient<Channel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GrpcServiceClient {{ }}")
    }
}
