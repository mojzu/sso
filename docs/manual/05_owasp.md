# Design

## OWASP: ASVS

The OWASP Application Security Verification Standard is being used as a reference to improve this application. These are some development and design notes based on requirements from the 4.0 version of the ASVS standard.

### 1.2.1

- Binary sso and postgres must be run as unique or special low privelege operating system accounts.
- Systemd unit file examples, sso, postgres, nginx, etc. users.

### 1.2.2

- HTTP calls (except ping) require service key authentication.
- Mutual TLS using rustls configuration and PKI.

### 1.2.3

- Binary sso is designed to provide multiple authentication mechanisms (none vetted yet).
- Relies on libraries which may be unvetted, e.g. libreauth, jsonwebtoken, rustls, etc.
- What does strong authentication include? Is it/can it be supported by this application?
- Primary use case for application is providing email/password login, which is probably not considered strong authentication.
- Audit logging and monitoring via prometheus are supported, need improvement for detecting account abuse or breaches.

### 1.2.4

- All authentication pathways are designed to be as strong as that pathway can be.
- For example, email password resets are supported which are probably not considered strong (can be disabled per user).

### 1.4.1

- All access controls are enforced at a trusted enforcement point (the server).
- Registered services must implement their own access controls for their own data.

### 1.4.2

- Access controls are designed for many services and many users, where users have access to one or more services.
- All registered services can see all users, but other services and keys belonging to them are hidden.
- Registered services may implement more fine grained access controls for their own data.

### 1.4.3

- Verify enforcement of principle of least privelege, requires more integration tests.

### 1.4.4

- HTTP calls (except ping) require service key authentication.
- Service key authentication mechanism code is split into a few functions, possible refactoring?

### 1.4.5

- Binary sso is designed to provide user authentication, is this out of scope?
