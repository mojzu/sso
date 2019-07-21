# Changelog

## 0.3.0 (unreleased)

**Added**

- Added email HTML templates, removed template parameters from routes.
- Defined server routes in `server::api` module.
- Finished client methods.
- Added support for audit list cases `gt AND lt`, `created_gte AND created_lte`.
- Added `/v1/metrics` endpoint for Prometheus integration.

**Changed**

- Changed audit list query parameters when using `created_gte` or `created_lte` options, added optional `offset_id` parameter to exclude previous results.
- Moved email handling from `server::smtp` module to `notify` module.
- Moved `main.rs:Error` into `cli` module, refactored error handling.
- Use `Forwarded` header instead of `X-Forwarded-For` for audit logs.
- Rename local authentication provider reset, update routes.

**Fixed**

- Reset password route returns OK in cases where user email address does not exist.

## 0.2.0 (2019-07-07)

**Added**

- Added audit logging to authentication routes.
- Added is enabled flag to services, users and keys, added is revoked flag to keys.
- Added synchronous and asynchronous clients to library.
- Added `email_eq` query option to user list route.
- Added local provider update user email, password routes.
- Added type parameter to service URL callback queries.

**Changed**

- Changed database schema to use strings as IDs.
- Changed list route endpoints to return data ID arrays.
- Changed CSRF key, value pairs to use TTL in seconds.
- Changed token type handling, user token split into access and refresh tokens.
- Moved OAuth2 routes into provider groups.
- Moved login, reset routes into local provider group.
- Upgraded to version 3 of `oauth2` library.
- Removed `skeptic` from tests, use Cargo test runner.
- Moved route types to public api module.

**Fixed**

- Fixed not validating service URLs before save using URL parse, improved URL error handling.
- Fixed inconsistent core error display strings.
- Fixed duplicate user email address returned internal server error code.
- Fixed Lettre email error type handling.

## 0.1.0 (2019-05-16)

- First version.
