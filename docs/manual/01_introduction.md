% sso 0.7.0
% Sam Ward
%

# Manual

## Introduction

**Warning: The author of this application is not a security expert, and the code has not undergone any kind of review or verification. Use it at your own risk!**

This crate **sso** is an authentication server. It is designed for use as a backend for other applications which must authenticate user requests, such as an API server.

Applications are registered as a **Service** with sso, a **Service Key** authenticates requests from the service to sso.

Users are registered as a **User** with sso, a **User Key** allows a user to authenticate with a service.

Authentication methods are organised into **Provider** groups.

### Local Provider

#### Password Login

- User can authenticate to a service using a unique email address and password.
- Login returns time-limited, revokable access and refresh [JSON web tokens][jwt].
- Access tokens are used to authenticate user requests to a service.
- Refresh tokens are used to produce new access and refresh tokens.
- User password can be reset via email, this feature can be disabled for a user.
- User can be required to update their passwords.
- User email address and password updates cause notification emails to be sent to the user.
- Notification emails contain revokation links in case a login is compromised.
- Passwords are stored as [argon2][argon2] hashes using [libreauth][libreauth].
- Password strength is checked by [zxcvbn][zxcvbn].
- Password leaks are checked by [Pwned Passwords][pwned-passwords].
- Users can be created without passwords to disable password login.

#### API Key

- Users can authenticate requests to a service using a unique, random key.
- Keys are not time-limited, but are revokable.

#### TOTP

- Services can authenticate user requests that contain a [TOTP][totp] code.

### GitHub Provider

#### OAuth2

- User can authenticate to a service using [GitHub OAuth2][github-oauth2].
- Login returns time-limited, revokable access and refresh [JSON web tokens][jwt].
- Access tokens are used to authenticate user requests to a service.
- Refresh tokens are used to produce new access and refresh tokens.

### Microsoft Provider

#### OAuth2

- User can authenticate to a service using [Microsoft OAuth2][microsoft-oauth2].
- Login returns time-limited, revokable access and refresh [JSON web tokens][jwt].
- Access tokens are used to authenticate user requests to a service.
- Refresh tokens are used to produce new access and refresh tokens.

## Definitions

The following terms are used throughout this manual.

### Service

An application that must authenticate user requests, registered as a service with `sso` using a name and callback URLs for supported authentication providers.

### User

A person or other external API consumer identified by a unique email address who wants to interact with one or more services registered with `sso`.

### Key

Random, unique keys produced by `sso` with a specified type (`Key`, `Token` or `Totp`). Keys may be revoked to prevent use.

### Root Key

Keys linked to no services or users, produced by command line and can be used to manage `sso` via HTTP requests. `Key` type.

### Service Key

Keys linked to one service, used by `sso` to authenticate HTTP requests from services. `Key` type.

### User Key

Keys linked to one user and one service, allows user to authenticate with the linked service. Any type, type determines what the key may be used for (e.g. `Token` keys are used for email and password login).

### Token

[JSON web tokens][jwt] used for time-limited authentication, produced for users from a user key. Revoking the key used to produce a token also revokes the token. Tokens have a type which determines what the token may be used for (e.g. single use reset password token).
