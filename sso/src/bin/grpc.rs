//! # Single Sign-On (gRPC Server)
//!
//! ## Environment Variables
//!
//! ### SSO_SENTRY_DSN
//!
//! Sentry DSN for logging, error handling integration, optional.
//!
//! ### SSO_LOG_PRETTY
//!
//! Format logs as multi-line JSON, optional, defaults to false.
//!
//! ### SSO_POSTGRES_URL
//!
//! Postgres connection URL, required.
//!
//! ### SSO_POSTGRES_CONNECTIONS
//!
//! Postgres connections, optional.
//!
//! ### SSO_USER_AGENT
//!
//! HTTP client user agent header, optional, defaults to `sso`.
//!
//! ### SSO_PWNED_PASSWORDS
//!
//! Pwned Passwords integration enabled, optional, defaults to false.
//!
//! ### SSO_TRAEFIK
//!
//! Traefik forward authentcation integration enabled, optional, defaults to false.
//!
//! ### SSO_SMTP_HOST
//!
//! SMTP server host, optional.
//!
//! ### SSO_SMTP_PORT
//!
//! SMTP server port, optional.
//!
//! ### SSO_SMTP_USER
//!
//! SMTP server user, optional.
//!
//! ### SSO_SMTP_PASSWORD
//!
//! SMTP server password, optional.
//!
//! ### SSO_SMTP_FILE
//!
//! SMTP file transport directory path, optional, defaults to `./tmp`.
//!
//! ### SSO_GITHUB_CLIENT_ID
//!
//! GitHub OAuth2 provider client ID, optional.
//!
//! ### SSO_GITHUB_CLIENT_SECRET
//!
//! GitHub OAuth2 provider client secret, optional.
//!
//! ### SSO_MICROSOFT_CLIENT_ID
//!
//! Microsoft OAuth2 provider client ID, optional.
//!
//! ### SSO_MICROSOFT_CLIENT_SECRET
//!
//! Microsoft OAuth2 provider client secret, optional.
//!
use futures_util::future::join;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use sso::{log_init, Postgres};
use std::sync::Arc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging, error handling.
    let _guard = log_init("SSO_SENTRY_DSN", "SSO_LOG_PRETTY");

    // Postgres connection.
    let driver = Postgres::from_env("SSO_POSTGRES_URL", "SSO_POSTGRES_CONNECTIONS");

    // gRPC server options.
    let options =
        sso::grpc::ServerOptions::from_env("SSO_USER_AGENT", "SSO_PWNED_PASSWORDS", "SSO_TRAEFIK")
            .smtp_transport_from_env(
                "SSO_SMTP_HOST",
                "SSO_SMTP_PORT",
                "SSO_SMTP_USER",
                "SSO_SMTP_PASSWORD",
            )
            .smtp_file_transport_from_env("SSO_SMTP_FILE")
            .github_from_env("SSO_GITHUB_CLIENT_ID", "SSO_GITHUB_CLIENT_SECRET")
            .microsoft_from_env("SSO_MICROSOFT_CLIENT_ID", "SSO_MICROSOFT_CLIENT_SECRET");
    let traefik_enabled = options.traefik_enabled();

    let sso = sso::grpc::Server::new(driver, options);
    let sso_ref = Arc::new(sso.clone());

    // gRPC server.
    let grpc = {
        let addr = "0.0.0.0:7042".parse()?;
        let svc = sso::grpc::SsoServer::new(sso);
        Server::builder().add_service(svc).serve(addr)
    };

    // HTTP server.
    let http = {
        let addr = "0.0.0.0:7043".parse()?;
        hyper::Server::bind(&addr).serve(make_service_fn(move |socket: &AddrStream| {
            let sso_ref = sso_ref.clone();
            let remote_addr = socket.remote_addr();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    sso::grpc::http_server(sso_ref.driver(), traefik_enabled, req, remote_addr)
                }))
            }
        }))
    };

    // Wait to exit gracefully.
    let (grpc, http) = join(grpc, http).await;
    grpc?;
    http?;
    Ok(())
}
