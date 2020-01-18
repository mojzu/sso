% sso 0.9.0
% Sam Ward
%

# Manual

## Introduction

**Warning: The author of this application is not a security expert. The code has not undergone any kind of review. Use it at your own risk!**

**sso** is an authentication server. It is designed for use as a backend for other applications which must authenticate user requests, such as API servers.

![Overview of Authentication System](docs/asset/introduction.svg)

1. **sso** authentication server.
2. Providers are registered as a **Service** with sso, for example API servers.
3. A **Service Key** is used to authenticate requests from the service to sso.
4. Consumers are registered as a **User** with sso.
5. A **User Key** is used to authenticate a user, and authenticate requests from the user to the service.

In the diagram above, `Service A` can authenticate requests from `User 1` and `User 2`. `Service B` can authenticate requests from `User 2` and `User 3`.

## Features

### Authentication

User authentication methods are organised into **Provider** groups. Services are registered with callback URLs for each supported provider.

#### Local Provider

User authentication using unique email address and password.

- User login returns access and refresh tokens.
- User registration with email confirmation.
- User password reset via email.
- User password update required.
- User email address and password updates require current password.
- Outgoing emails contain revokation links to disable user access in case of compromised access.
- Password stored as [argon2][argon2] hash using [libreauth][libreauth].
- Password strength checked by [zxcvbn][zxcvbn].
- Password leaks checked by [Pwned Passwords][pwned-passwords].
- Password not set disables password login.
- User key for service of `Token` type is required.

#### GitHub Provider

User authentication using [GitHub OAuth2][github-oauth2].

- User login returns access and refresh tokens.
- User key for service of `Token` type is required.

#### Microsoft Provider

User authentication using [Microsoft OAuth2][microsoft-oauth2].

- User login returns access and refresh tokens.
- User key for service of `Token` type is required.

#### Key

Request authentication using an API key distributed by the service.

- User authenticates requests to a service using a unique, random key.
- User key can be revoked, but is not time-limited.
- User key for service of `Key` type is required.

#### Token

Request authentication using access token returned by user authentication provider, for example local login.

- User authenticates requests to a service using a [JWT][jwt] access token.
- User generates new access and refresh tokens using a [JWT][jwt] refresh token.
- User token is time-limited.
- User key can be revoked, which also revokes all tokens the key produced.
- User key for service of `Token` type is required.

#### TOTP

Request authentication using [TOTP][totp] code generated from a key distributed by the service.

- User key for service of `Totp` type is required.

### CSRF Tokens

Services can use **sso** to create and verify single-use [CSRF tokens][csrf]

- If service uses cookies for authentication, these tokens are used in form templates to prevent CSRF attacks.

### Audit Logging

All **sso** endpoint failures after input validation are audited. `POST`, `PATCH`, `DELETE` endpoint successes are also audited.

- Services are able to read, create and update their own audit logs.
- Audit logs are append only, logs can be created when requests are received and response data can be added when request handled.
- Audit logs have retention time (default 3 months).
