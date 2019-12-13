use std::sync::Arc;
use tonic::{body::BoxBody, transport::Server};
use tower::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:9090".parse()?;
    let sso = sso_grpc::SsoGrpc::new();
    let sso_ref = Arc::new(sso.clone());

    Server::builder()
        .interceptor_fn(move |svc, req| {
            // let auth_header = req.headers().get("authorization").clone();

            // println!("{:?}", req);

            // let authed = if let Some(auth_header) = auth_header {
            //     auth_header == "Bearer some-secret-token"
            // } else {
            //     false
            // };

            let path_intercept = sso_ref.path_interceptor(req.uri().path());
            let fut = svc.call(req);

            async move {
                match path_intercept {
                    Ok(Some(res)) => {
                        drop(fut);
                        Ok(res)
                    }
                    Ok(None) => fut.await,
                    Err(e) => {
                        drop(fut);
                        Ok(http::Response::builder()
                            .status(500)
                            .header("grpc-status", format!("{}", e.code() as isize))
                            .header("grpc-message", e.message())
                            .body(BoxBody::empty())
                            .unwrap())
                    }
                }
            }
        })
        .add_service(sso_grpc::pb::server::SsoServer::new(sso))
        .serve(addr)
        .await?;

    Ok(())
}
