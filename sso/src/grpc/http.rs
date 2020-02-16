use crate::{grpc::util::*, *};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::{net::SocketAddr, sync::Arc};

static NOT_FOUND: &[u8] = b"NotFound";
static PONG: &[u8] = b"Pong";

/// HTTP server request handler for internal endpoints.
pub async fn http_server(
    driver: Arc<Postgres>,
    traefik_enabled: bool,
    req: Request<Body>,
    remote: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/ping") => ping(req).await,
        (&Method::GET, "/metrics") => metrics(driver, req).await,
        (&Method::GET, "/hook/traefik/self") => {
            if traefik_enabled {
                traefik_self(driver, req, remote).await
            } else {
                // Return 401 unauthorised response.
                Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap())
            }
        }
        (&Method::GET, "/hook/traefik/service") => {
            if traefik_enabled {
                traefik_service(driver, req, remote).await
            } else {
                // Return 401 unauthorised response.
                Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(Body::empty())
                    .unwrap())
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

// TODO(sam,feature): Traefik integration, example for service integration.
// Other headers: x-forwarded-host, x-forwarded-uri, x-real-ip.

async fn traefik_self(
    driver: Arc<Postgres>,
    req: Request<Body>,
    remote: SocketAddr,
) -> Result<Response<Body>, hyper::Error> {
    // Authorization
    let authorisation = if let Some(x) = req.headers().get(HEADER_AUTHORISATION) {
        match x.to_str() {
            Ok(x) => pattern::HeaderAuth::parse_key(x),
            Err(_e) => None,
        }
    } else {
        None
    };
    // User-Authorization
    let user_authorisation = if let Some(x) = req.headers().get(HEADER_USER_AUTHORISATION) {
        match x.to_str() {
            Ok(x) => pattern::HeaderAuth::parse(x),
            Err(_e) => None,
        }
    } else {
        None
    };
    // User-Agent
    let user_agent = if let Some(x) = req.headers().get("user-agent") {
        match x.to_str() {
            Ok(x) => x,
            Err(_e) => "none",
        }
    } else {
        "none"
    };
    // X-Forwarded-For
    let forwarded = if let Some(x) = req.headers().get("x-forwarded-for") {
        match x.to_str() {
            Ok(x) => Some(x.to_owned()),
            Err(_e) => None,
        }
    } else {
        None
    };
    let remote = format!("{}", remote);
    let (audit_meta, auth) = (
        AuditMeta::new(user_agent, remote, forwarded, user_authorisation),
        authorisation,
    );

    let driver = driver.clone();
    let data = blocking::<_, MethodError, _>(move || {
        audit_result_err(
            driver.as_ref(),
            audit_meta,
            AuditType::Traefik,
            |driver, audit| {
                pattern::key_authenticate(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;
                Ok((
                    audit.get_key_id(),
                    audit.get_service_id(),
                    audit.get_user_key_id(),
                    audit.get_user_id(),
                ))
            },
        )
    })
    .await;

    let res = match data {
        Ok((key_id, service_id, user_key_id, user_id)) => {
            let mut builder = Response::builder().status(StatusCode::OK);
            if let Some(key_id) = key_id {
                builder = builder.header("Grpc-Metadata-Sso-Key-Id", key_id.to_string());
            }
            if let Some(service_id) = service_id {
                builder = builder.header("Grpc-Metadata-Sso-Service-Id", service_id.to_string());
            }
            if let Some(user_key_id) = user_key_id {
                builder = builder.header("Grpc-Metadata-Sso-User-Key-Id", user_key_id.to_string());
            }
            if let Some(user_id) = user_id {
                builder = builder.header("Grpc-Metadata-Sso-User-Id", user_id.to_string());
            }

            builder.body(Body::empty()).unwrap()
        }
        Err(_e) => Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Body::empty())
            .unwrap(),
    };
    Ok(res)
}

async fn traefik_service(
    _driver: Arc<Postgres>,
    req: Request<Body>,
    _remote: SocketAddr,
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
