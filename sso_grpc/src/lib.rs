use tonic::{body::BoxBody, Request, Response, Status};
// use bytes::IntoBuf;

pub mod pb {
    //! Generated protobuf server and client items.
    tonic::include_proto!("sso");
}

use pb::{AuditListReply, AuditListRequest, Empty, PingReply};

/// Metrics URL path.
pub const URL_PATH_METRICS: &str = "/metrics";

/// OpenAPI JSON URL path.
pub const URL_PATH_OPENAPI_JSON: &str = "/openapi.json";

/// gRPC server.
#[derive(Clone)]
pub struct SsoGrpc {}

impl SsoGrpc {
    /// Returns a new `SsoGrpc`.
    pub fn new() -> Self {
        Self {}
    }

    /// Returns some `http::Response` in case path matches a known route.
    pub fn path_interceptor(&self, path: &str) -> Result<Option<http::Response<BoxBody>>, Status> {
        match path {
            // // TODO(refactor): Implement this.
            // URL_PATH_METRICS => {
            //     let b = "blah blah blah".to_owned().into_buf();
            //     Ok(Some(http::Response::builder()
            //     .status(200)
            //     .header("grpc-status", "0")
            //     .body(BoxBody::new(b))
            //     .unwrap()))
            // }
            _ => Ok(None),
        }
    }
}

#[tonic::async_trait]
impl pb::server::Sso for SsoGrpc {
    async fn ping(&self, request: Request<Empty>) -> Result<Response<PingReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = PingReply {
            pong: format!("Hello!"),
        };

        Ok(Response::new(reply))
    }

    async fn audit_list(
        &self,
        request: Request<AuditListRequest>,
    ) -> Result<Response<AuditListReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = AuditListReply {};

        Ok(Response::new(reply))
    }
}
