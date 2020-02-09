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
//! ### SSO_DATABASE_URL
//!
//! Database connection URL, required.
//!
//! ### SSO_DATABASE_CONNECTIONS
//!
//! Database connections, required.
//!
//! ### SSO_PASSWORD_PWNED
//!
//! Password pwned integration enabled, optional, defaults to false.
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
use hyper::service::{make_service_fn, service_fn};
use sso::{env, log_init, Postgres};
use std::{fs::create_dir_all, sync::Arc};
use tonic::transport::Server;

// /// Server TLS certificate file.
// const ENV_TLS_CERT_PEM: &str = "SSO_TLS_CERT_PEM";
// /// Server TLS key file.
// const ENV_TLS_KEY_PEM: &str = "SSO_TLS_KEY_PEM";
// /// Server mutual TLS client file.
// const ENV_TLS_CLIENT_PEM: &str = "SSO_TLS_CLIENT_PEM";

// TODO(sam,refactor): TLS support, blocked on `ring-asm`.
// <https://github.com/hyperium/tonic/blob/master/examples/src/tls/server.rs>
// <https://github.com/smallstep/autocert>

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging, error handling.
    let _guard = log_init("SSO_SENTRY_DSN", "SSO_LOG_PRETTY");

    // Database connection.
    // TODO(sam,refactor): Improve initialisation code, expect, etc.
    let database_url = env::string("SSO_DATABASE_URL").unwrap();
    let database_connections = env::value_opt::<u32>("SSO_DATABASE_CONNECTIONS").unwrap();
    let driver = Postgres::initialise(&database_url, database_connections).unwrap();

    let password_pwned = env::value_opt::<bool>("SSO_PASSWORD_PWNED")
        .unwrap()
        .unwrap_or(false);
    let github_oauth2 = env::oauth2("SSO_GITHUB_CLIENT_ID", "SSO_GITHUB_CLIENT_SECRET").unwrap();
    let microsoft_oauth2 =
        env::oauth2("SSO_MICROSOFT_CLIENT_ID", "SSO_MICROSOFT_CLIENT_SECRET").unwrap();

    let smtp = env::smtp(
        "SSO_SMTP_HOST",
        "SSO_SMTP_PORT",
        "SSO_SMTP_USER",
        "SSO_SMTP_PASSWORD",
    )
    .unwrap();
    // Create directory for SMTP file transport if other variables are undefined.
    let smtp_file = "./tmp".to_owned();
    create_dir_all(&smtp_file)?;

    let options = sso::grpc::ServerOptions::new("sso", password_pwned)
        .smtp_transport(smtp)
        .smtp_file_transport(Some(smtp_file))
        .github(github_oauth2)
        .microsoft(microsoft_oauth2);
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
        hyper::Server::bind(&addr).serve(make_service_fn(move |_| {
            let sso_ref = sso_ref.clone();
            async {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    sso::grpc::http_server(sso_ref.driver(), req)
                }))
            }
        }))
    };

    let (grpc, http) = join(grpc, http).await;
    grpc?;
    http?;
    Ok(())
}
