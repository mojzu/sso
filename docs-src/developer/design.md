# Design

## Dependencies

Crate dependencies for cryptography, authentication.

- [libreauth][libreauth]
- [jsonwebtoken][jsonwebtoken]
- [oauth2-rs][oauth2-rs]

## OWASP: ASVS

[OWASP ASVS][owasp-asvs]

The OWASP Application Security Verification Standard is being used as a reference to improve this application. These are some development and design notes based on requirements from the 4.0 version of the ASVS standard. This is a self-evaluation and should be viewed skeptically.

### 1.2.1

- Binaries must be run as unique or special low privilege operating system accounts.

### 1.2.2

- HTTP calls (except ping) require service key authentication.

### 1.2.3

- Server is designed to provide multiple authentication mechanisms, none of which have been vetted.
- Relies on libraries which may be unvetted, e.g. libreauth, jsonwebtoken, rustls, etc.
- What does strong authentication mean in this context?
- One feature is providing email/password login, which is probably not considered strong authentication.
- Audit logging and monitoring via prometheus.

### 1.2.4

- All authentication pathways are designed to be as strong as that pathway can be.
- For example, email password resets are supported which are probably not considered strong.

### 1.4.1

- All access controls are enforced at a trusted enforcement point (the server).
- Registered services must implement their own access controls for their own data.

### 1.4.2

- Access controls are designed for many services and many users, where users have access to one or more services.
- All registered services can read all registered users, other services and keys belonging to them are hidden.
- Registered services may implement more complex access controls for their own data.

### 1.4.3

- Verify enforcement of principle of least privelege, requires more integration tests.

### 1.4.4

- HTTP calls (except ping) require service key authentication.

### 1.4.5

- This crate provides user authentication, not access control, is this out of scope?

### 1.5.1

- Terms of service columns and user accepted columns, handle changes to terms.

### 1.5.2

- API is JSON requests over HTTP so serialisation is required.
- Using [serde][serde] for serialisation and deserialisation.

### 1.5.3

- Input validation is enforced at a trusted enforcement point (the server).
- Using [validator][validator] for input validation.

### 1.5.4

- All output encoding is [UTF-8][utf-8].

### 1.6.1

- Key values are used for [JWT][jwt] cryptographic encoding and decoding.
- Key values are only returned to service or user on creation.
- Keys can be disabled and/or revoked to prevent use.

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

### 1.8.1, 1.8.2

- Sensitive data is not identified or classified into protection levels.

### 1.9.1, 1.9.2

- Connection to database, other services must be encrypted.

### 1.10.1

- Git and GitHub used for source code control, no formal commit procedure.

### 1.11.1

- Little documentation and definitions of application components, out of date.

### 1.11.2, 1.11.3

- No unsynchronised state shared between high-value business logic flows.
- All code should be threadsafe (no use of unsafe).
- I don't think there is any but probably needs more thorough check, including dependencies.

### 1.12.1, 1.12.2

- No user uploaded files, if feature added in future files will be stored as binary blobs in database.
- Serve files as octet stream downloads if added.

[libreauth]: https://docs.rs/libreauth/0.12.0/libreauth/
[jsonwebtoken]: https://github.com/Keats/jsonwebtoken
[oauth2-rs]: https://github.com/ramosbugs/oauth2-rs
[owasp-asvs]: https://www.owasp.org/index.php/Category:OWASP_Application_Security_Verification_Standard_Project
