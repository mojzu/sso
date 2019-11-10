% sso 0.8.0
% Sam Ward
%

# Manual

## Introduction

**Warning: The author of this application is not a security expert. The code has not undergone any kind of review. Use it at your own risk!**

**sso** is an authentication server. It is designed for use as a backend for other applications which must authenticate user requests, such as API servers.

![Overview of Authentication System](docs/asset/introduction.svg)

1. **sso** authentication server.
2. Applications are registered as a **Service** with sso.
3. A **Service Key** is used to authenticate requests from the service to sso.
4. Users are registered as a **User** with sso.
5. A **User Key** allows a user to authenticate with a service.

In the diagram above, service A can authenticate requests from users 1 and 2. Service B can authenticate requests from users 2 and 3.

### Providers

User authentication methods are organised into **Provider** groups. Services are registered with callback URLs for each supported provider.

#### Local Provider

##### Password Login

```
POST /v1/auth/provider/local/login
POST /v1/auth/provider/local/register
POST /v1/auth/provider/local/register/confirm
POST /v1/auth/provider/local/reset-password
POST /v1/auth/provider/local/reset-password/confirm
POST /v1/auth/provider/local/update-email
POST /v1/auth/provider/local/update-email/revoke
POST /v1/auth/provider/local/update-password
POST /v1/auth/provider/local/update-password/revoke
```

- User authenticate with a service using a unique email address and password.
- Login returns access and refresh tokens.
- User registration is supported with email confirmation.
- User password may be reset via email, this feature can be disabled for a user.
- User can be required to update their passwords.
- User email address and password updates require current password, an email is sent to the user to notify them of the change.
- Emails to user contain revokation links in case a login is compromised.
- Passwords are stored as [argon2][argon2] hashes using [libreauth][libreauth].
- Password strength is checked by [zxcvbn][zxcvbn].
- Password leaks are checked by [Pwned Passwords][pwned-passwords].
- Users may be created without passwords to disable password login.
- User key for service of `Token` type is required.

#### GitHub Provider

##### OAuth2

```
GET,POST /v1/auth/provider/github/oauth2
```

- User authenticates to a service using [GitHub OAuth2][github-oauth2].
- Login returns access and refresh tokens.
- User key for service of `Token` type is required.

#### Microsoft Provider

##### OAuth2

```
GET,POST /v1/auth/provider/microsoft/oauth2
```

- User authenticates to a service using [Microsoft OAuth2][microsoft-oauth2].
- Login returns access and refresh tokens.
- User key for service of `Token` type is required.

### Authentication

After successful user authentication, further requests from the user are authenticated with the following methods.

#### Key

```
POST /v1/auth/key/verify
POST /v1/auth/key/revoke
```

- Service can distribute API keys to users.
- Users authenticate requests to a service using a unique, random key.
- Keys can be revoked, but are not time-limited.
- User key for service of `Key` type is required.

#### Token

```
POST /v1/auth/token/verify
POST /v1/auth/token/refresh
POST /v1/auth/token/revoke
```

- Tokens are returned by other authentication methods (e.g. login, OAuth2).
- Users authenticate requests to a service using a [JWT][jwt] access token.
- Users generate new access and refresh tokens using a [JWT][jwt] refresh token.
- Tokens can be revoked, and are time-limited.
- Revoking the key used to produce a token also revokes the token.
- User key for service of `Token` type is required.
- Tokens have an internal type which determines what the token is used for.

#### TOTP

```
POST /v1/auth/totp
```

- Services authenticate user requests that contain a [TOTP][totp] code.
- User key for service of `Totp` type is required.
