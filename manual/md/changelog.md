# Changelog

## 0.2.0 (unreleased)

### Added

- Add active flag to users.
- Add synchronous and asynchronous clients.

### Changed

- Changed list route endpoints to return data ID arrays.
- Reorganise OAuth2 routes into provider groups.
- Reorganise login, reset routes into local provider group.

### Fixed

- Validate service URLs before save using URL parse, improved URL error handling.
- Fix inconsistent core error display strings.
- Fix duplicate user email address returned internal server error code.
- Fix Lettre email error type handling.

---

## 0.1.0 (2019-05-16)

- First version.
