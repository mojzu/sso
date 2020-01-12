#[macro_use]
extern crate log;

use futures_util::future::join;
use sentry::integrations::log::LoggerOptions;
use sso::{env, pattern, Driver, DriverPostgres};
use std::{fs::create_dir_all, sync::Arc};
use tonic::transport::Server;
use tower::Service;

/// Sentry URL for logging integration.
const ENV_SENTRY_URL: &str = "SSO_SENTRY_URL";

/// Database connection URL.
const ENV_DATABASE_URL: &str = "SSO_DATABASE_URL";
/// Database connection.
const ENV_DATABASE_CONNECTIONS: &str = "SSO_DATABASE_CONNECTIONS";

/// Server TLS certificate file.
const ENV_TLS_CERT_PEM: &str = "SSO_TLS_CERT_PEM";
/// Server TLS key file.
const ENV_TLS_KEY_PEM: &str = "SSO_TLS_KEY_PEM";
/// Server mutual TLS client file.
const ENV_TLS_CLIENT_PEM: &str = "SSO_TLS_CLIENT_PEM";

/// SMTP server, optional.
const ENV_SMTP_HOST: &str = "SSO_SMTP_HOST";
const ENV_SMTP_PORT: &str = "SSO_SMTP_PORT";
const ENV_SMTP_USER: &str = "SSO_SMTP_USER";
const ENV_SMTP_PASSWORD: &str = "SSO_SMTP_PASSWORD";

/// Password pwned integration enabled, optional.
const ENV_PASSWORD_PWNED: &str = "SSO_PASSWORD_PWNED";

/// GitHub OAuth2 provider, optional.
const ENV_GITHUB_CLIENT_ID: &str = "SSO_GITHUB_CLIENT_ID";
const ENV_GITHUB_CLIENT_SECRET: &str = "SSO_GITHUB_CLIENT_SECRET";

/// Microsoft OAuth2 provider, optional.
const ENV_MICROSOFT_CLIENT_ID: &str = "SSO_MICROSOFT_CLIENT_ID";
const ENV_MICROSOFT_CLIENT_SECRET: &str = "SSO_MICROSOFT_CLIENT_SECRET";

// TODO(refactor): TLS support, blocked on `ring-asm`.
// <https://github.com/hyperium/tonic/blob/master/examples/src/tls/server.rs>
// <https://github.com/smallstep/autocert>

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // If SENTRY_URL is defined, enable logging and panic handler integration.
    // TODO(feature): Log in JSON, use fluentd to forward to Sentry.
    let _guard = match std::env::var(ENV_SENTRY_URL) {
        Ok(sentry_url) => {
            let guard = sentry::init(sentry_url);
            let mut options = LoggerOptions::default();
            options.emit_warning_events = true;

            sentry::integrations::env_logger::init(None, options);
            sentry::integrations::panic::register_panic_handler();
            Some(guard)
        }
        Err(e) => {
            env_logger::init();
            warn!("SENTRY_URL is undefined, integration is disabled ({})", e);
            None
        }
    };

    // Setup database connection.
    let database_url = env::string(ENV_DATABASE_URL).unwrap();
    let database_connections = env::value_opt::<u32>(ENV_DATABASE_CONNECTIONS).unwrap();
    let driver = DriverPostgres::initialise(&database_url, database_connections)
        .unwrap()
        .box_clone();

    // Start background task thread.
    let (task_handle, task_tx) =
        pattern::task_thread_start(driver.clone(), 1000, &chrono::Duration::weeks(12));

    let password_pwned = env::value_opt::<bool>(ENV_PASSWORD_PWNED)
        .unwrap()
        .unwrap_or(false);
    let github_oauth2 = env::oauth2(ENV_GITHUB_CLIENT_ID, ENV_GITHUB_CLIENT_SECRET).unwrap();
    let microsoft_oauth2 =
        env::oauth2(ENV_MICROSOFT_CLIENT_ID, ENV_MICROSOFT_CLIENT_SECRET).unwrap();

    let smtp = env::smtp(
        ENV_SMTP_HOST,
        ENV_SMTP_PORT,
        ENV_SMTP_USER,
        ENV_SMTP_PASSWORD,
    )
    .unwrap();
    // Create directory for SMTP file transport if other variables are undefined.
    let smtp_file = "./tmp".to_owned();
    create_dir_all(&smtp_file)?;

    let grpc_addr = "0.0.0.0:7042".parse()?;
    let options = sso::grpc::ServerOptions::new("sso", password_pwned)
        .smtp_transport(smtp)
        .smtp_file_transport(Some(smtp_file))
        .github(github_oauth2)
        .microsoft(microsoft_oauth2);
    let sso = sso::grpc::Server::new(driver, options);
    let sso_ref = Arc::new(sso.clone());

    let grpc_sso = sso_ref.clone();
    let grpc = Server::builder()
        .interceptor_fn(move |svc, req| {
            let metrics = grpc_sso.metrics(req.uri().path());
            let fut = svc.call(req);
            async move {
                match fut.await {
                    Ok(res) => {
                        metrics.end(res.status().as_u16());
                        Ok(res)
                    }
                    Err(e) => {
                        metrics.end(1);
                        Err(e)
                    }
                }
            }
        })
        .add_service(sso::grpc::pb::sso_server::SsoServer::new(sso))
        .serve(grpc_addr);

    let http_addr = "0.0.0.0:7043".parse()?;
    let http_sso = sso_ref.clone();
    let http = hyper::Server::bind(&http_addr).serve(hyper::service::make_service_fn(move |_| {
        let sso_ref = http_sso.clone();
        async {
            Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                sso::grpc::http_response(sso_ref.driver(), req)
            }))
        }
    }));

    let (grpc, http) = join(grpc, http).await;
    grpc?;
    http?;

    pattern::task_thread_stop(task_handle, task_tx);
    Ok(())
}
