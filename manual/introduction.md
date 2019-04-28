# Introduction

**Warning: The author of this application is not a security export, nor has the code undergone any kind of review or verification. Use at your own risk.**

Ark Auth is an authentication server, it is intended to be used as a backend for other services which must authenticate their users requests. The following authentication methods are supported.

## Authentication Methods

### Password Login

- Users can authenticate to services using a unique email address and password.
- Logins produce a time-limited, revokable json web token which can be used to authenticate requests.
- User passwords can be reset via email.
- Passwords are stored as `bcrypt` hashes.
- Password strength is checked by `zxcvbn`.
- Password leaks are checked by `haveibeenpwned.com`.
- Users can be created without passwords to disable password login.

### API Key

- API consumers can authenticate requests to services using a unique, random key.
- Keys are not time-limited but are revokable.

### OAuth2

- Users can authenticate to services via an OAuth2 provider.
- Authentication produces a time-limited, revokable json web token which can be used to authenticate requests.

## Overview

The following diagram illustrates how services and Ark Auth integrate to authenticate user requests.

![User request verification](./images/diagram.svg)

1. User with token or key sends HTTP request to service.
2. Service sends HTTP request to Ark Auth with the users token or key, and its own key.
3. Ark Auth authenticates service using the provided key, and verifies user token or key.
4. If authenticated/verified, service handles request and sends HTTP response to user.
5. User handles HTTP response.

TODO(doc)
