## 0.3.0 (unreleased)

**Added**

- Added email HTML templates, removed template parameters from routes.
- Defined server routes in `server::api` module.
- Finished client methods.
- Added support for audit list cases `gt AND lt`, `created_gte AND created_lte`.
- Added `/v1/metrics` endpoint for Prometheus integration.
- Added support for optional audit logs to some authentication endpoints.

**Changed**

- Changed audit list query parameters when using `created_gte` or `created_lte` options, added optional `offset_id` parameter to exclude previous results.
- Moved email handling from `server::smtp` module to `notify` module.
- Moved `main.rs:Error` into `cli` module, refactored error handling.
- Use `Forwarded` header instead of `X-Forwarded-For` for audit logs.
- Rename local authentication provider reset, update routes.
- Improved configuration interface using `derive_builder` crate.
- Added hostname configuration to server.
- Switched to `rustls` crate for TLS support in Actix, `ssl` feature still used for client HTTPS requests.

**Fixed**

- Reset password route returns OK in cases where user email address does not exist.
- Audit log errors created for root key authentication when checking if service key.
