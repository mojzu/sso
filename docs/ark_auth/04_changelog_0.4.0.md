## 0.4.0 (unreleased)

**Added**

- Added some process metrics to Prometheus integration.
- Created asynchronous client actor to handle outgoing HTTP requests.
- Added environment variable `PASSWORD_PWNED_ENABLED` to enable pwned passwords integration.

**Changed**

- Renamed configuration structures to options to improve consistency.
- Improved server, client modules public interfaces.

**Fixed**

- Fixed missing `dyn` keyword warnings.
