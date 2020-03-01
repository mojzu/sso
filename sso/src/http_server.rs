use crate::prelude::*;
use hyper::{Body, Method, Request, Response, StatusCode};
use std::{net::SocketAddr, sync::Arc};

static NOT_FOUND: &[u8] = b"NotFound";
static PONG: &[u8] = b"Pong";

/// HTTP server.
#[derive(Debug)]
pub struct HttpServer;

impl HttpServer {
    /// Request handler for internal endpoints.
    pub async fn handler(
        options: Arc<GrpcServerOptions>,
        driver: Arc<Postgres>,
        remote: SocketAddr,
        req: Request<Body>,
    ) -> Result<Response<Body>, hyper::Error> {
        match (req.method(), req.uri().path()) {
            (&Method::GET, "/ping") => ping(req).await,
            (&Method::GET, "/metrics") => metrics(driver, req).await,
            (&Method::GET, "/hook/traefik/self") => {
                if options.traefik_enabled() {
                    traefik_self(driver, req, remote).await
                } else {
                    // Return 401 unauthorised response.
                    Ok(response_unauthorised())
                }
            }
            (&Method::GET, "/hook/traefik/service") => {
                if options.traefik_enabled() {
                    traefik_service(driver, req, remote).await
                } else {
                    // Return 401 unauthorised response.
                    Ok(response_unauthorised())
                }
            }
            _ => {
                // Return 404 not found response.
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(NOT_FOUND.into())
                    .unwrap())
            }
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
    let s = blocking_hyper(move || Ok(Metrics::read(driver.as_ref()).unwrap())).await?;
    Ok(Response::new(Body::from(s)))
}

async fn traefik_self(
    driver: Arc<Postgres>,
    req: Request<Body>,
    remote: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {
    let remote = format!("{}", remote);
    let (audit_meta, auth) = (
        AuditMeta::from_header_map(req.headers(), remote),
        HeaderAuth::from_header_map(req.headers(), false),
    );

    let driver = driver.clone();
    let audit_builder = blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::Traefik,
            |driver, audit| {
                pattern::key_authenticate(driver, audit, &auth)
                    .map_err(GrpcMethodError::Unauthorised)?;
                Ok(audit.clone())
            },
        )
    })
    .await;

    Ok(match audit_builder {
        Ok(audit) => response_from_audit_builder(audit),
        Err(_e) => response_unauthorised(),
    })
}

async fn traefik_service(
    driver: Arc<Postgres>,
    req: Request<Body>,
    remote: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {
    let remote = format!("{}", remote);
    let (audit_meta, auth) = (
        AuditMeta::from_header_map(req.headers(), remote),
        HeaderAuth::from_header_map(req.headers(), false),
    );
    let service_key = header_service_authorisation(req.headers());

    let driver = driver.clone();
    let audit_builder = blocking_method(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::Traefik,
            |driver, audit| {
                pattern::user_key_token_authenticate(driver, audit, &auth, service_key.clone())
                    .map_err(GrpcMethodError::Unauthorised)?;
                Ok(audit.clone())
            },
        )
    })
    .await;

    Ok(match audit_builder {
        Ok(audit) => response_from_audit_builder(audit),
        Err(_e) => response_unauthorised(),
    })
}

fn response_unauthorised() -> Response<Body> {
    let grpc_status = format!("{}", tonic::Code::Unauthenticated as u8);
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Grpc-Status", grpc_status)
        .header("Grpc-Message", ERR_REDACTED)
        .body(Body::empty())
        .unwrap()
}

fn response_from_audit_builder(audit: AuditBuilder) -> Response<Body> {
    let mut builder = Response::builder().status(StatusCode::OK);
    if let Some(key_id) = audit.get_key_id() {
        builder = builder.header(HEADER_GRPC_METADATA_SSO_KEY_ID, key_id.to_string());
    }
    if let Some(service_id) = audit.get_service_id() {
        builder = builder.header(HEADER_GRPC_METADATA_SSO_SERVICE_ID, service_id.to_string());
    }
    if let Some(user_key_id) = audit.get_user_key_id() {
        builder = builder.header(
            HEADER_GRPC_METADATA_SSO_USER_KEY_ID,
            user_key_id.to_string(),
        );
    }
    if let Some(user_id) = audit.get_user_id() {
        builder = builder.header(HEADER_GRPC_METADATA_SSO_USER_ID, user_id.to_string());
    }
    builder.body(Body::empty()).unwrap()
}
