//! # sso
//!
#[macro_use]
extern crate paperclip_macros;
#[macro_use]
extern crate log;
#[macro_use]
extern crate postgres_types;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate validator_derive;

mod cli;
mod client;
mod common;
mod config;
mod error;
mod internal;
mod mailto;
pub mod oauth2;
mod postgres;
mod server;
mod util;

pub use crate::{
    cli::*, client::*, config::*, error::*, mailto::*, postgres::*, server::*, util::*,
};

// Todo: Fix
// - Metrics counters incrementing on each collection?

// Todo: Test
// - OWASP tests
// - Register fails when disabled for client
// - What happens to flows when codes/etc. expire? E.g. test register and reload page

// Todo: Refactor
// - Check master branch for lost features during rewrite
// - Rust client for integration
// - Postgres improve query to/from helper structs

// Todo: Feature
// - OWASP research
//     - https://cheatsheetseries.owasp.org
// - Oauth2 research, OIDC, other RFC support?
//     - https://oauth.net/2/oauth-best-practice/
//     - https://oauth.net/2/
//     - https://openid.net/specs/openid-connect-core-1_0.html
//     - https://oauth.net/2/pkce/
//     - https://github.com/ramosbugs/openidconnect-rs
// - User stat table support
// - Translations based on locale
// - Confirm emails for reset/update/etc.
// - Variable length secrets? api keys
// - Token naming from request headers?
// - Email/password updates when using oauth2?
// - Replace reqwest with surf?
// - Improved prometheus metrics, labelling
// - Prometheus alert definitions
// - Opentelemetry Jaeger/other integration?
//     - <https://github.com/open-telemetry/opentelemetry-rust/blob/master/examples/actix-http/src/main.rs>
// - Fluentd/sentry integration?
// - Backup/restore functionality for config/database, cron job?
//     - NAME-VERSION-TIMESTAMP format, rolling or incremental/encryption, error/alert in case of failure
//     - Output to file, pg_dump for database, restore, migration/legacy support
// - Update management/safety
//     - <https://docs.rs/tokio-postgres/0.5.5/tokio_postgres/struct.Client.html#method.transaction>
// - Runtime config changes support, for example to log level
// - Error serialisation for log messages, additional info in panics
// - TeamCity deployment, .deb, docker images?
// - Unit/integration tests with postgres connection? Mocking libraries? pgTAP?
// - Email HTML templates
// - Pwned password support somehow, pass back as header in oauth2 flow? oidc related?
// - Audit retention, dump deleted logs to file?
// - Email verification/accepting terms
// - User metadata
// - Possible JWT usage for ids/keys/etc., oidc related?
// - PKCE support
// - Forward authentication support (caddy, nginx?)
//     - Set client ID header on requests
//     - SSO endpoints checks for auth cookie
//     - If access denied authorize and redirect back to client
// - Docker image upload and size labels (+ other labels?)
//     - <https://shields.io/category/size>
