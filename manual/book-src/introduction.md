# Introduction

## Authentication Methods

Authentication methods are organised into provider groups, based on the provider, the following user authentication methods are supported.

### Password Login (Local)

- Users can authenticate to a service using a unique email address and password.
- A successful login produces time-limited, revokable access and refresh tokens which can be used to authenticate user requests and to produce new tokens.
- User passwords can be reset via email.
- User password is required to update email address or password, if updated an email is sent to the user with a revokation link in case of compromised access.
- Passwords are stored as `bcrypt` hashes.
- Password strength is checked by `zxcvbn`.
- Password leaks are checked by `haveibeenpwned.com`.
- Users can be created without passwords to disable password login.

### API Key (Local)

- Users can authenticate requests to a service using a unique, random key.
- Keys are not time-limited but are revokable.

### OAuth2 (GitHub, Microsoft)

- Users can authenticate to services via supported OAuth2 providers.
- Successful authentication by OAuth2 server produces time-limited, revokable access and refresh tokens which can be used to authenticate user requests and to produce new tokens.

## Definitions

Definitions for terms used throughout this manual.

### User

A person or API consumer identified by a unique email address who wants to interact with one or more services.

### Service

An application that must authenticate user requests, registered as a service with Ark Auth using a name and callback URL.

### Key

Random, unique keys produced by Ark Auth. Keys may be revoked to prevent use.

#### Root Key

Keys linked to no services or users, produced by command line and can be used to manage Ark Auth via HTTP requests.

#### Service Key

Keys linked to one service, used by Ark Auth to authenticate HTTP requests from services.

#### User Key

Keys linked to one user and one service, allows user to authenticate with the linked service.

### Token

JSON web tokens used for time-limited authentication, produced for users from a user key. Revoking the key used to produce a token also revokes the token.
