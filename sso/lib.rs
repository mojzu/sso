//! # sso
//!
#![recursion_limit = "1024"]
#![type_length_limit = "65536"]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unused_variables)]
#![warn(clippy::all)]

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

pub mod cli;
mod common;
pub mod config;
mod error;
pub mod http_client;
pub mod http_server;
mod internal;
pub mod mailto;
pub mod metrics;
pub mod oauth2;
pub mod openid;
mod postgres;
pub mod util;
pub mod validate;

pub use crate::error::*;

// todo: Fix
// - Metrics counters incrementing on each collection?

// todo: Refactor
// todo: Check master branch for lost features during rewrite
// - Postgres improve query to/from helper structs
// - Check for panicable code (unwrap, etc.), rewrite and test
// - Register template strings in Handlebars
// - Use `_audit: &mut Audit` variables
// - Use lettre alpha for async mailto?

// todo: Test
// - OWASP tests
// - Rust client integration test
// - Register fails when disabled for client
// - What happens to flows when codes/etc. expire? E.g. test register and reload page
// - Password reset tests with multiple clients, cannot reuse tokens, etc.
// - Benchmarks/profiling tools and support? flamegraph?
// - More tests for data access, is client data masked correctly
// - Test requests with other/unknown content types are handled correctly
// - More input tests including unicode passwords, bad strings, etc.
// - User password flag tests (reset, update required etc)

// todo: Feature
// - Pwned password support somehow, pass back as header in oauth2 flow? oidc related?
//     - Config setting to enable/disable
// - Docker default compose for running example (part of manual)?
// - Confirm emails for reset/update/etc.
// - Token naming from request headers?
// - Email/password updates when using oauth2?
// - Forward authentication support (caddy, nginx, traefik?)
//     - Set client ID header on requests
//     - SSO endpoints checks for auth cookie
//     - If access denied authorize and redirect back to client
// - PKCE support
// - Matrix client oauth2, other integrations testing?
// - Github Oauth2 provider support
//     - More generic Oauth2 provider support, method of configuration?
// - Expose more SMTP transport options?
// - CORS support for API requests, or docs on how to add at proxy layer?
//     - OWASP secure headers project
// - TLS support/testing, client authentication?
// - OWASP research, OWASP ASVS reading and notes
//     - https://cheatsheetseries.owasp.org
// - Oauth2 research, OIDC, other RFC support?
//     - https://oauth.net/2/oauth-best-practice/
//     - https://oauth.net/2/
//     - https://openid.net/specs/openid-connect-core-1_0.html
//     - https://oauth.net/2/pkce/
//     - https://github.com/ramosbugs/openidconnect-rs
// - User stat table support
//     - Last login, token uses information
// - Translations based on locale
// - Use timezones in user communications
// - Replace reqwest with surf?
// - Improved prometheus metrics, labelling
//     - https://prometheus.io/
// - Prometheus alert definitions/examples
// - Opentelemetry Jaeger/other integration?
//     - https://github.com/open-telemetry/opentelemetry-rust/blob/master/examples/actix-http/src/main.rs
// - Fluentd/sentry logging integration? DSN configuration
// - Backup/restore functionality for config/database, cron job?
//     - NAME-VERSION-TIMESTAMP format, rolling or incremental/encryption, error/alert in case of failure
//     - Output to file, pg_dump for database, restore, migration/legacy support
//     - Using diesel migrations or other mechanism for changes over time?
// - Update management/safety
//     - https://docs.rs/tokio-postgres/0.5.5/tokio_postgres/struct.Client.html#method.transaction
// - Runtime config changes support, for example to log level
// - Error serialisation for log messages, additional info in panics
// - Email HTML templates, translation/formatting using user locale and timezone
// - Audit retention, dump deleted logs to file? Audit log metrics and prometheus alert rules?
// - Email verification/accepting terms
// - User metadata, may require access controls
// - Possible JWT usage for ids/keys/etc., oidc related?
//     - https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_Cheat_Sheet_for_Java.html
// - Rate limiting support, or at proxy level?
// - Kubernetes examples/support/integration, also systemd examples?
// - Token revokation support, totp (libreauth) support/options? webauthn? sqrl?
// - Improved OpenAPI mapping interface, possibility of using hyper/tower?
// - Client scopes to limit sso server access (forward auth/trusted/untrusted?)
// - CLI improvements and documentation
// - Key management standards research
//     - https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-57pt1r4.pdf
// - Evaluate data and identity/classify sensitive data
// - Mutual TLS encryption/auth for postgres connection
// - Some kind of formalised procedures around source code changes.
// - IP allowlists/other security features built into server? SMS alert support?
// - Production/development mode flags to disable some features?
//     - e.g. localhost or https only?
// - Password update cannot set same password, is this in line with recommendations?
// - User sessions route, other HTML interfaces? GUI service example?
// - Client/user groups for segmentation
// - GDPR and other data protection compliance research
// - Constant time responses for authentication endpoints to resist timing attacks
// - Embeddable services/support for integrations?
