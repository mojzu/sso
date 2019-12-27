use crate::{grpc::blocking, *};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::sync::Arc;

static NOT_FOUND: &[u8] = b"Not Found";
static PONG: &[u8] = b"Pong";

pub async fn http_response(
    driver: Arc<Box<dyn Driver>>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/ping") => ping(req).await,
        (&Method::GET, "/metrics") => metrics(driver, req).await,
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
    driver: Arc<Box<dyn Driver>>,
    _req: Request<Body>,
) -> Result<Response<Body>, hyper::Error> {
    let driver = driver.clone();
    let s = blocking::<_, hyper::Error, _>(move || {
        Ok(Metrics::read(driver.as_ref().as_ref(), None).unwrap())
    })
    .await?;
    Ok(Response::new(Body::from(s)))
}
