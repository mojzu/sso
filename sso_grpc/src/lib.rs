pub mod pb {
    //! Generated protobuf server and client items.
    tonic::include_proto!("sso");
}
mod client;

pub use crate::{client::{ClientBlocking, ClientOptions}, pb::sso_client::SsoClient as Client};

use crate::pb::{AuditListReply, AuditListRequest, Empty, Text};
use tonic::{
    body::{Body, BoxBody},
    Request, Response, Status,
};

/// gRPC server.
#[derive(Clone)]
pub struct SsoGrpc {}

impl SsoGrpc {
    /// Returns a new `SsoGrpc`.
    pub fn new() -> Self {
        Self {}
    }

    // /// Returns some `http::Response` in case path matches a known route.
    // pub fn path_interceptor(&self, path: &str) -> Result<Option<http::Response<BoxBody>>, Status> {
    //     match path {
    //         // TODO(refactor): Implement this.
    //         URL_PATH_METRICS => {
    //             let b = bytes::Bytes::from("blah blah blah");
    //             let bo = BoxBody::new(b);
    //             let r = http::Response::builder()
    //             .status(200)
    //             .header("grpc-status", "0")
    //             .body(bo)
    //             .unwrap();
    //             Ok(Some(r))
    //         }
    //         _ => Ok(None),
    //     }
    // }
}

#[tonic::async_trait]
impl pb::sso_server::Sso for SsoGrpc {
    async fn ping(&self, request: Request<Empty>) -> Result<Response<Text>, Status> {
        println!("Got a request: {:?}", request);

        let reply = Text {
            text: format!("Hello!"),
        };

        Ok(Response::new(reply))
    }

    async fn metrics(&self, request: Request<Empty>) -> Result<Response<Text>, Status> {
        Ok(Response::new(Text {
            text: "# prometheus".to_owned(),
        }))
    }

    // async fn audit_list(
    //     &self,
    //     request: Request<AuditListRequest>,
    // ) -> Result<Response<AuditListReply>, Status> {
    //     println!("Got a request: {:?}", request);

    //     let reply = AuditListReply {};

    //     Ok(Response::new(reply))
    // }
}
