# Changelog

## 0.7.0 (?)

### Added

- Added `Debug` trait implementations for public types.
- Added filters to key list endpoint with `KeyFilter` options.
- Added filters to service list endpoint with `ServiceFilter` options.
- Added `id` query parameter array for key, service and user list endpoint filtering.

### Changed

- Changed table column names to remove prefixes for `DriverPostgres`.

### Fixed

- Fixed `clippy` too many arguments warnings.
- Fixed `ClientActor` GET request ignores query string.
- Fixed 500 status code returned trying to create key with invalid service or user ID.

## 0.6.0 (2019-10-07)

### Changed

- Replaced key flags with type: `Key`, `Token` or `Totp`.
- Added user flags: `password_allow_reset` and `password_require_update`.
- Return `Unauthorised` instead of `Forbidden` for missing authentication.
- User update email and password endpoints accept `user_id` instead of `key` or `token`.

### Fixed

- Fixed read user by ID route `/v1/user/{id}` attempts to read key.

## 0.5.0 (2019-10-05)

### Added

- Added TOTP validation route.
- Added key flags: `allow_key`, `allow_token`, `allow_totp`.
- Added user fields: `locale`, `timezone`.

### Changed

- Renamed project `sso`, ark was taken by various products and crates.
- Code consistency improvements.
- Renamed audit `path` to `type` string.
- Replaced `serde_urlencoded` with `serde_qs` for more advanced query string support.
- Improved audit metric collection efficiency.
- Changed authentication provider interface so callbacks always go to service.

## 0.4.0 (2019-09-09)

### Added

- Added some process metrics to Prometheus integration.
- Created asynchronous client actor to handle outgoing HTTP requests.
- Added environment variable `PASSWORD_PWNED_ENABLED` to enable pwned passwords integration.

### Changed

- Replaced `bcrypt` with `libreauth` crate for password hashing.
- Use `libreauth` key generation for CSRF keys and `Key` values.
- Improved modules public interfaces.
- Changed driver interface to use `Uuid` types instead of strings.
- Renamed configuration structures to options to improve consistency.

### Fixed

- Fixed missing `dyn` keyword warnings.

## 0.3.0 (2019-08-03)

### Added

- Improved email templates, removed template parameters from routes.
- Defined server routes and other constants in `server::api` module.
- Finished synchronous and asynchronous clients.
- Added support for audit list query cases `gt AND lt`, `created_gte AND created_lte`.
- Added `/v1/metrics` endpoint for Prometheus integration.
- Added support for optional audit logs to some authentication endpoints.
- Added TLS configuration support to server and clients.
- Added hostname configuration to server.

### Changed

- Changed audit list query parameters when using `created_gte` or `created_lte` options, added optional `offset_id` parameter to exclude previous results.
- Moved email handling from `server::smtp` module to `notify` module.
- Moved `main.rs:Error` into `cli` module, refactored error handling.
- Use `Forwarded` header instead of `X-Forwarded-For` for audit logs.
- Renamed local authentication provider reset, update routes.
- Improved configuration interfaces using `derive_builder` crate.

### Fixed

- Reset password route now returns OK in cases where user email address does not exist.
- Fixed audit log errors were created for root key authentication when authenticating a service key.
- Fixed internal server errors returned to client when OAuth2 provider is disabled.

## 0.2.0 (2019-07-07)

### Added

- Added audit logging to authentication routes.
- Added is enabled flag to services, users and keys, added is revoked flag to keys.
- Added synchronous and asynchronous clients to library.
- Added `email_eq` query option to user list route.
- Added local provider update user email, password routes.
- Added type parameter to service URL callback queries.

### Changed

- Changed database schema to use strings as IDs.
- Changed list route endpoints to return data ID arrays.
- Changed CSRF key, value pairs to use TTL in seconds.
- Changed token type handling, user token split into access and refresh tokens.
- Moved OAuth2 routes into provider groups.
- Moved login, reset routes into local provider group.
- Upgraded to version 3 of `oauth2` library.
- Removed `skeptic` from tests, use Cargo test runner.
- Moved route types to public api module.

### Fixed

- Fixed not validating service URLs before save using URL parse, improved URL error handling.
- Fixed inconsistent core error display strings.
- Fixed duplicate user email address returned internal server error code.
- Fixed Lettre email error type handling.

## 0.1.0 (2019-05-16)

### Added

- First version.
