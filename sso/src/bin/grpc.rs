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
#[macro_use]
extern crate log;

use futures_util::future::join;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use sso::{log_init, GrpcServer, GrpcServerOptions, HttpServer, Postgres};
use std::{sync::Arc, io};
use tonic::transport::Server;

async fn signal_terminate() -> io::Result<()> {
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?.recv().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging, error handling.
    let _guard = log_init("SSO_SENTRY_DSN", "SSO_LOG_PRETTY");

    // Postgres connection.
    let driver = Postgres::from_env("SSO_POSTGRES_URL", "SSO_POSTGRES_CONNECTIONS");

    // gRPC, HTTP server options.
    let grpc_options =
        GrpcServerOptions::from_env("SSO_USER_AGENT", "SSO_PWNED_PASSWORDS", "SSO_TRAEFIK")
            .smtp_transport_from_env(
                "SSO_SMTP_HOST",
                "SSO_SMTP_PORT",
                "SSO_SMTP_USER",
                "SSO_SMTP_PASSWORD",
            )
            .smtp_file_transport_from_env("SSO_SMTP_FILE")
            .github_from_env("SSO_GITHUB_CLIENT_ID", "SSO_GITHUB_CLIENT_SECRET")
            .microsoft_from_env("SSO_MICROSOFT_CLIENT_ID", "SSO_MICROSOFT_CLIENT_SECRET");
    let http_options = Arc::new(grpc_options.clone());

    let sso = GrpcServer::new(driver, grpc_options);
    let http_sso = Arc::new(sso.clone());

    // gRPC server.
    let grpc = {
        let addr = "0.0.0.0:7042".parse()?;
        info!("Listening on grpc://{}", addr);
        Server::builder()
            .add_service(sso.service())
            .serve_with_shutdown(addr, async {
                signal_terminate()
                    .await
                    .expect("Graceful shutdown failure.");
            })
    };

    // HTTP server.
    let http = {
        let addr = "0.0.0.0:7043".parse()?;
        info!("Listening on http://{}", addr);
        let server =
            hyper::Server::bind(&addr).serve(make_service_fn(move |socket: &AddrStream| {
                let options = http_options.clone();
                let sso = http_sso.clone();
                let remote_addr = socket.remote_addr();
                async move {
                    Ok::<_, hyper::Error>(service_fn(move |req| {
                        HttpServer::handler(options.clone(), sso.driver(), remote_addr, req)
                    }))
                }
            }));
        server.with_graceful_shutdown(async {
            signal_terminate()
                .await
                .expect("Graceful shutdown failure.");
        })
    };

    // Wait to exit gracefully.
    let (grpc, http) = join(grpc, http).await;
    grpc?;
    http?;

    Ok(())
}
