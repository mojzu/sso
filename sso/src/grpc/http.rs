use crate::{grpc::util::*, *};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::sync::Arc;

static NOT_FOUND: &[u8] = b"NotFound";
static PONG: &[u8] = b"Pong";

/// HTTP server request handler for internal endpoints.
pub async fn http_server(
    driver: Arc<Postgres>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/ping") => ping(req).await,
        (&Method::GET, "/metrics") => metrics(driver, req).await,
        (&Method::GET, "/hook/traefik/self") => traefik_self(driver, req).await,
        (&Method::GET, "/hook/traefik/service") => traefik_service(driver, req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOT_FOUND.into())
                .unwrap())
        }
    }
}

async fn ping(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from(PONG)))
}

async fn metrics(
    driver: Arc<Postgres>,
    _req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let driver = driver.clone();
    let s = hyper_blocking(move || Ok(Metrics::read(driver.as_ref()).unwrap())).await?;
    Ok(Response::new(Body::from(s)))
}

// TODO(sam,feature): Traefik integration, optional flag, example for service integration.

async fn traefik_self(
    _driver: Arc<Postgres>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    info!("traefik_self {:?}", req.headers());
    let res = Response::builder()
        .status(StatusCode::OK)
        .header("Grpc-Metadata-Sso-Key-Id", "key-id-test")
        .header("Grpc-Metadata-Sso-Service-Id", "service-id-test")
        .body(Body::empty())
        .unwrap();
    Ok(res)
}

async fn traefik_service(
    _driver: Arc<Postgres>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    info!("traefik_service {:?}", req.headers());
    let res = Response::builder()
        .status(StatusCode::OK)
        .header("Grpc-Metadata-Sso-Key-Id", "key-id-test")
        .header("Grpc-Metadata-Sso-User-Id", "user-id-test")
        .body(Body::empty())
        .unwrap();
    Ok(res)
}
