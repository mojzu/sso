mod method;
mod server;

pub use crate::grpc_service::server::*;

use crate::grpc::pb::sso_service_client::SsoServiceClient;
use tonic::transport::Channel;

/// gRPC service asynchronous client.
pub type GrpcServiceClient = SsoServiceClient<Channel>;
