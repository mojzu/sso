#[macro_use]
extern crate log;

use futures_util::future::join;
use sentry::integrations::log::LoggerOptions;
use sso::{env, pattern, Cli, CliOptions, Driver, DriverPostgres};
use std::sync::Arc;
use tonic::{body::BoxBody, transport::Server};
use tower::Service;

/// Sentry URL for logging integration.
const ENV_SENTRY_URL: &str = "SSO_GRPC_SENTRY_URL";

/// Database connection URL.
const ENV_DATABASE_URL: &str = "SSO_GRPC_DATABASE_URL";
/// Database connection.
const ENV_DATABASE_CONNECTIONS: &str = "SSO_GRPC_DATABASE_CONNECTIONS";

/// Server bind address (gRPC).
const ENV_BIND: &str = "SSO_GRPC_BIND";
/// Server TLS certificate file.
const ENV_TLS_CERT_PEM: &str = "SSO_GRPC_TLS_CERT_PEM";
/// Server TLS key file.
const ENV_TLS_KEY_PEM: &str = "SSO_GRPC_TLS_KEY_PEM";
/// Server mutual TLS client file.
const ENV_TLS_CLIENT_PEM: &str = "SSO_GRPC_TLS_CLIENT_PEM";
/// Server bind address (HTTP).
const ENV_HTTP_BIND: &str = "SSO_GRPC_HTTP_BIND";

/// SMTP server, optional.
const ENV_SMTP_HOST: &str = "SSO_GRPC_SMTP_HOST";
const ENV_SMTP_PORT: &str = "SSO_GRPC_SMTP_PORT";
const ENV_SMTP_USER: &str = "SSO_GRPC_SMTP_USER";
const ENV_SMTP_PASSWORD: &str = "SSO_GRPC_SMTP_PASSWORD";
/// Write emails to files in directory, optional.
/// If server settings are defined this setting is ignored.
const ENV_SMTP_FILE: &str = "SSO_GRPC_SMTP_FILE";

/// Password pwned integration enabled, optional.
const ENV_PASSWORD_PWNED: &str = "SSO_GRPC_PASSWORD_PWNED";

/// GitHub OAuth2 provider, optional.
const ENV_GITHUB_CLIENT_ID: &str = "SSO_GRPC_GITHUB_CLIENT_ID";
const ENV_GITHUB_CLIENT_SECRET: &str = "SSO_GRPC_GITHUB_CLIENT_SECRET";

/// Microsoft OAuth2 provider, optional.
const ENV_MICROSOFT_CLIENT_ID: &str = "SSO_GRPC_MICROSOFT_CLIENT_ID";
const ENV_MICROSOFT_CLIENT_SECRET: &str = "SSO_GRPC_MICROSOFT_CLIENT_SECRET";

// TODO(refactor): TLS support.
// <https://github.com/hyperium/tonic/blob/master/examples/src/tls/server.rs>

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure logging environment variables.
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("RUST_LOG", "info");

    // If SENTRY_URL is defined, enable logging and panic handler integration.
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

    let database_url = env::string(ENV_DATABASE_URL).unwrap();
    let database_connections = env::value_opt::<u32>(ENV_DATABASE_CONNECTIONS).unwrap();
    let driver = DriverPostgres::initialise(&database_url, database_connections)
        .unwrap()
        .box_clone();

    let options = CliOptions::new();
    let (task_handle, task_tx) = pattern::task_thread_start(
        driver.clone(),
        options.task_tick_ms(),
        options.audit_retention(),
    );

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
    let smtp_file = env::string_opt(ENV_SMTP_FILE);

    let bind = env::string(ENV_BIND).unwrap();
    let grpc_addr = bind.parse()?;
    let options = sso::grpc::ServerOptions::new("sso")
        .smtp_transport(smtp)
        .smtp_file_transport(smtp_file);
    let sso = sso::grpc::Server::new(driver, options);
    let sso_ref = Arc::new(sso.clone());

    let grpc_sso = sso_ref.clone();
    let grpc = Server::builder()
        .interceptor_fn(move |svc, req| {
            let p = req.uri().path().to_owned();
            let (t, c) = grpc_sso.metrics_start(&p);

            let fut = svc.call(req);

            let sso_ref = grpc_sso.clone();
            async move {
                let res = fut.await;
                sso_ref.metrics_end(t, c, &p, "0");
                res
            }
        })
        .add_service(sso::grpc::pb::sso_server::SsoServer::new(sso))
        .serve(grpc_addr);

    let http_bind = env::string(ENV_HTTP_BIND).unwrap();
    let http_addr = http_bind.parse()?;
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

// TODO(refactor): Metrics end status collection.
// let auth_header = req.headers().get("authorization").clone();
// println!("{:?}", req);
// let authed = if let Some(auth_header) = auth_header {
//     auth_header == "Bearer some-secret-token"
// } else {
//     false
// };
// let path_intercept = sso_ref.path_interceptor(req.uri().path());
// match path_intercept {
//     Ok(Some(res)) => {
//         drop(fut);
//         Ok(res)
//     }
//     Ok(None) => fut.await,
//     Err(e) => {
//         drop(fut);
//         Ok(http::Response::builder()
//             .status(500)
//             .header("grpc-status", format!("{}", e.code() as isize))
//             .header("grpc-message", e.message())
//             .body(BoxBody::empty())
//             .unwrap())
//     }
// }
