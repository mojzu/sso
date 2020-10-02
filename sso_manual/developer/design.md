# Design

-  [OWASP - Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)

## Client Secrets

-   [The Client ID and Secret](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/)

Client secrets are 256 bit random keys generated using the pgcrypto extension. The client ID and secret produce a SHA256 HMAC, which is stored in the configuration file.

The client authenticates requests to the server using HTTP basic authentication, the server checks that the client ID and secret HMAC match the stored HMAC.

Secrets are generated using the `sso._secret_generate` SQL function, HMACs are generated using `sso._secret_hash` and checked using `sso._secret_check`. These functions are defined in `postgres/setup.sql` and available as functions in the `Postgres` module.

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

## Sessions

-   [OWASP - Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)

## Tokens

User access and refresh tokens are 256 bit random keys generated using the pgcrypto extension. The token ID and access or refresh secret produce a SHA256 HMAC, which is AES encrypted with the client secret.

User access and refresh tokens can be used with the OAuth2 introspection endpoint, the server decrypts the token using the requesting clients secret key, and then checks the token ID and secret HMAC were produced using the token access or refresh secret.

This method is different to client secret checks which is inconsistent, however these should probably be some kind of JWT instead for OIDC support.

-   [OWASP - JSON Web Token Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)

Secrets are generated using the `sso._secret_generate` SQL function, HMACs are generated using `sso._secret_hash`, tokens are encrypted using `sso._secret_encrypt`, tokens are decrypted using `sso._secret_decrypt` and checked using `sso._secret_check`.

## API Keys

-   [OWASP - REST Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html)

API keys are 256 bit random keys generated using the pgcrypto extension. The API key ID and secret produce a SHA256 HMAC, which is stored in the database.

User API keys can be verified with the API key verify endpoint, the server checks the API key ID and secret HMAC match the stored HMAC.

Secrets are generated using the `sso._secret_generate` SQL function, HMACs are generated using `sso._secret_hash` and checked using `sso._secret_check`.
