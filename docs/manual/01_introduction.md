% Manual (mz_auth)
% Sam Ward
%

# Introduction

**Warning: The author of this application is not a security expert, and the code has not undergone any kind of review or verification. Use it at your own risk!**

The crate `mz_auth` is an authentication server binary. It is intended to be used as a backend for other services, such as API servers, which must authenticate user requests.

Authentication methods are organised into provider groups. The following provider groups are currently supported: `local`, `github`, `microsoft`. Depending on the provider, the following user authentication methods are supported.

- **Password Login (Local)**
  - Users can authenticate to a service using a unique email address and password.
  - Successful login returns time-limited, revokable access and refresh [JSON web tokens](https://jwt.io/).
  - Access tokens are used to authenticate user requests.
  - Refresh tokens are used to produce new access and refresh tokens.
  - User passwords can be reset via email.
  - User email address and password updates cause notification emails to be sent to the user.
  - Notification emails contain revokation links in case a login is compromised.
  - Passwords are stored as [argon2](https://en.wikipedia.org/wiki/Argon2) hashes.
  - Password strength is checked by [zxcvbn](https://github.com/shssoichiro/zxcvbn-rs).
  - Password leaks are checked by [Pwned Passwords](pwned-passwords).
  - Users can be created without passwords to disable password login.
- **API Key (Local)**
  - Users can authenticate requests to a service using a unique, random key.
  - Keys are not time-limited, but are revokable.
- **OAuth2 (GitHub, Microsoft)**
  - Users can authenticate to a service via supported OAuth2 providers.
  - Successful login returns time-limited, revokable access and refresh [JSON web tokens](https://jwt.io/).
  - Access tokens are used to authenticate user requests.
  - Refresh tokens are used to produce new access and refresh tokens.

## Definitions

The following terms are used throughout this manual.

- **User**
  - A person or other external API consumer identified by a unique email address who wants to interact with one or more services registered with `mz_auth`.
- **Service**
  - An application that must authenticate user requests, registered as a service with `mz_auth` using a name and callback URL.
- **Key**
  - Random, unique keys produced by `mz_auth`. Keys may be revoked to prevent use.
- **Root Key**
  - Keys linked to no services or users, produced by command line and can be used to manage `mz_auth` via HTTP requests.
- **Service Key**
  - Keys linked to one service, used by `mz_auth` to authenticate HTTP requests from services.
- **User Key**
  - Keys linked to one user and one service, allows user to authenticate with the linked service.
- **Token**
  - [JSON web tokens](https://jwt.io/) used for time-limited authentication, produced for users from a user key. Revoking the key used to produce a token also revokes the token.
