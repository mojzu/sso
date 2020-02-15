# Design

## Dependencies

Crate dependencies for cryptography, authentication.

- [libreauth][libreauth]
- [jsonwebtoken][jsonwebtoken]
- [oauth2-rs][oauth2-rs]

## Possible Features

- TODO(sam,feature): User sessions route for active tokens/keys.
- TODO(sam,feature): Support more OAuth2 providers.
- TODO(sam,feature): [Webauthn][webauthn] support (libreauth?).
- TODO(sam,feature): Configurable canary routes.
- TODO(sam,feature): Improved public library API interface (gui service as example?).
- TODO(sam,feature): Email translation/formatting using user locale and timezone.
- TODO(sam,feature): Handle changes to password hash version.
- TODO(sam,feature): Option to enforce provider URLs HTTPS.
- TODO(sam,feature): User last login, key last use information (calculate in SQL).
- TODO(sam,feature): Login from unknown IP address warnings, SMS support?
- TODO(sam,feature): Service IP whitelist.
- TODO(sam,feature): Service/user groups for segmentation.
- TODO(sam,feature): Password update cannot set same password.

## OWASP: ASVS

[OWASP ASVS][owasp-asvs]

The OWASP Application Security Verification Standard is being used as a reference to improve this application. These are some development and design notes based on requirements from the 4.0 version of the ASVS standard. This is a self-evaluation and should be viewed skeptically.

### 1.2.1

- Binaries must be run as unique or special low privilege operating system accounts.
- TODO(sam,docs): Systemd unit file examples, sso, postgres, etc.
- TODO(sam,docs): Kubernetes deployment examples.

### 1.2.2

- HTTP calls (except ping) require service key authentication.
- TODO(sam,docs): Mutual TLS using rustls configuration and PKI for communication between services.

### 1.2.3

- Server is designed to provide multiple authentication mechanisms, none of which have been vetted.
- Relies on libraries which may be unvetted, e.g. libreauth, jsonwebtoken, rustls, etc.
- What does strong authentication mean in this context?
- One feature is providing email/password login, which is probably not considered strong authentication.
- Audit logging and monitoring via prometheus.
- TODO(sam,feature): Audit logging and prometheus metrics improvements for detecting account abuse and breaches.

### 1.2.4

- All authentication pathways are designed to be as strong as that pathway can be.
- For example, email password resets are supported which are probably not considered strong.
- TODO(sam,feature): More opt-ins for pathway branches that may be weak, for example ability to reset passwords by email.

### 1.4.1

- All access controls are enforced at a trusted enforcement point (the server).
- Registered services must implement their own access controls for their own data.

### 1.4.2

- Access controls are designed for many services and many users, where users have access to one or more services.
- All registered services can read all registered users, other services and keys belonging to them are hidden.
- Registered services may implement more complex access controls for their own data.
- TODO(sam,test): More tests for data access, is service data masked correctly.

### 1.4.3

- Verify enforcement of principle of least privelege, requires more integration tests.
- TODO(sam,test): More tests on preventing spoofing, elevation of privelege.

### 1.4.4

- HTTP calls (except ping) require service key authentication.
- TODO(sam,refactor): Service key authentication mechanism code is split across files, cleaner code.

### 1.4.5

- This crate provides user authentication, not access control, is this out of scope?
- TODO(sam,feature): Structured data for users, may require access controls.

### 1.5.1

- Terms of service columns and user accepted columns, handle changes to terms.
- TODO(sam,docs): GDPR and other data protection compliance research.

### 1.5.2

- API is JSON requests over HTTP so serialisation is required.
- Using [serde][serde] and [serde_qs][serde_qs] for serialisation and deserialisation.
- TODO(sam,test): Test requests with other/unknown content types are handled correctly.
- TODO(sam,feature): Flag(s) to require HTTPS to ensure all requests/responses are encrypted in transmit.

### 1.5.3

- Input validation is enforced at a trusted enforcement point (the server).
- Using [validator][validator] for input validation.
- TODO(sam,test): More input tests including unicode passwords, bad strings, etc.

### 1.5.4

- All output encoding is [UTF-8][utf-8].

### 1.6.1

- Key values are used for [JWT][jwt] cryptographic encoding and decoding.
- Key values are only returned to service or user on creation.
- Keys can be disabled and/or revoked to prevent use.
- TODO(sam,docs): Check this meets key management standard NIST SP 800-57.

### 1.6.2

- Cannot verify that services protect created key values.

### 1.6.3

- No hard-coded keys or passwords, all keys and passwords can be replaced.

### 1.6.4

- API key support is clear-text equivalent.
- Authentication via API key is probably not considered low risk secret.
- Keys can be disabled/revoked to mitigate breaches, but this is not a solution.

### 1.7.1, 1.7.2

- Audit log format is common and used when making calls via API.
- Stdout/stderr logging is not consistent.
- Audit logs are saved to table, not transmitted to a remote system.
- Stdout/stderr logging is not transmitted to a remote system,
- TODO(sam,refactor): Option to transmit audit logs, stdout/stderr to fluentd.

### 1.8.1, 1.8.2

- Sensitive data is not identified or classified into protection levels.
- TODO(sam,docs): Evaluate data and identify/classify sensitive data.

### 1.9.1, 1.9.2

- Connection to database, other services must be encrypted.
- TODO(sam,docs): Mutual TLS encryption/authentication for postgres connection.
- TODO(sam,refactor): Dependency updates, blocked on `jsonwebtoken`.

### 1.10.1

- Git and GitHub used for source code control, no formal commit procedure.
- TODO(sam,docs): Some kind of formalised procedures around source code changes.

### 1.11.1

- Little documentation and definitions of application components, out of date.
- TODO(sam,docs): Up to date documentation and definitions of application components, diagrams.

### 1.11.2, 1.11.3

- No unsynchronised state shared between high-value business logic flows.
- All code should be threadsafe (no use of unsafe).
- I don't think there is any but probably needs more thorough check, including dependencies.
- TODO(sam,feature): Constant time responses for authentication endpoints to resist timing attacks.

### 1.12.1, 1.12.2

- No user uploaded files, if feature added in future files will be stored as binary blobs in database.
- Serve files as octet stream downloads if added.
