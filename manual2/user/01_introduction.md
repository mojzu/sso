% ark_auth: User Manual
% Sam Ward
%

# Introduction

**Warning: The author of this application is not a security expert, nor has the code undergone any kind of review or verification. Use it at your own risk!**

`ark_auth` is an authentication server, it is intended to be used as a backend for services which must authenticate their users requests.

## Authentication Methods

Authentication methods are exposed via a [REST][rest] API, and are organised into provider groups. The following provider groups are currently supported: `local`, `github`, `microsoft`. Depending on the provider, the following user authentication methods are supported.

**Password Login (Local)**

Users can authenticate to a service using a unique email address and password. A successful login produces time-limited, revokable access and refresh [JSON web tokens][jwt], which can be used to authenticate user requests and produce new tokens. This method also has the following features.

- User passwords can be reset via email.
- User email address and password can be updated, notification emails containing revokation links are emailed to the user in case of compromised access.
- Passwords are stored as [Bcrypt][bcrypt] hashes.
- Password strength is checked by [zxcvbn][zxcvbn-rs].
- Password leaks are checked by [Pwned Passwords][haveibeenpwned].
- Users can be created without passwords to disable password login.

**API Key (Local)**

Users can authenticate requests to a service using a unique, random key. This must be distributed by the service and is not time-limited, although is revokable by the service in case of compromised access.

**OAuth2 (GitHub, Microsoft)**

Users can authenticate to services via supported OAuth2 providers. Successful authentication by OAuth2 server produces time-limited, revokable access and refresh [JSON web tokens][jwt], which can be used to authenticate user requests and produce new tokens.

## Definitions

The following terms are used throughout this manual.

**User**

A person or API consumer identified by a unique email address who wants to interact with one or more services registered with `ark_auth`.

**Service**

An application that must authenticate user requests, registered as a service with `ark_auth` using a name and callback URL.

**Key**

Random, unique keys produced by `ark_auth`. Keys may be revoked to prevent use.

**Root Key**

Keys linked to no services or users, produced by command line and can be used to manage `ark_auth` via HTTP requests.

**Service Key**

Keys linked to one service, used by `ark_auth` to authenticate HTTP requests from services.

**User Key**

Keys linked to one user and one service, allows user to authenticate with the linked service.

**Token**

[JSON web tokens][jwt] used for time-limited authentication, produced for users from a user key. Revoking the key used to produce a token also revokes the token.

[rest]: <https://en.wikipedia.org/wiki/Representational_state_transfer>
[jwt]: <https://jwt.io/>
[bcrypt]: <https://en.wikipedia.org/wiki/Bcrypt>
[zxcvbn-rs]: <https://github.com/shssoichiro/zxcvbn-rs>
[haveibeenpwned]: <https://haveibeenpwned.com/Passwords>
