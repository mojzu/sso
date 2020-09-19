# Design

## Client Secrets

-   [The Client ID and Secret](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/)

Client secrets are 256 bit random keys generated using the pgcrypto extension. The client ID and secret produce a SHA256 HMAC which is stored in the configuration file.

The client authenticates requests to the server using HTTP basic authentication, the server checks that the client ID and secret HMAC match the stored HMAC.

Secrets are generated using the `sso._secret_generate` SQL function, HMACs are generated using the `sso._secret_hash` SQL function and checked using the `sso._secret_check` SQL function. These functions are defined in `postgres/setup.sql` and available as functions in the `Postgres` module.

## User Passwords

-   [OWASP - Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)

User passwords are stored as Bcrypt hashes using the pgcrypto extension with a work factor of 12. User password hashes may be stored in the configuration file and the database.

Password length is validated by the server with a minimum of 8 and maximum of 64.

HTML client interface includes `zxcvbn` password strength check, this is only run by the client browser.

Plaintext passwords are not logged or audited by the server, and the code attempts to keep them in memory for as little time as required. But passwords are sent in plaintext to the database as a query parameter for hashing/checking, which could be a problem depending on how database logging is configured.

There is currently no method for upgrading legacy hashes.

Hashes are generated using the `sso._password_hash` SQL function, and are checked against a password using the `sso._password_check` SQL function. These functions are defined in `postgres/setup.sql` and available as functions in the `Postgres` module.

<!-- todo: Use peppering/pre-hashing? -->
<!-- todo: Legacy hash upgrade mechanism -->

-   [OWASP - Forgot Password Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html)

<!-- todo: Urls are https or localhost -->

## Tokens

<!-- todo: Tokens info/best practices -->

## API Keys

<!-- todo: API keys info/best practices -->
