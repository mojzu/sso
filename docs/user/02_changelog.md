# Changelog

## 0.2.0 (unreleased)

**Added**

- Added audit logging to authentication routes.
- Added is enabled flag to services, users and keys, added is revoked flag to keys.
- Added synchronous and asynchronous clients to library.
- Added `email_eq` query option to user list route.
- Added local provider update user email, password routes.

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
