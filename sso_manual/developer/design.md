# Design

-  [OWASP - Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)

## Client Secrets

-   [The Client ID and Secret](https://www.oauth.com/oauth2-servers/client-registration/client-id-secret/)

Client secrets are 256 bit random keys generated using the pgcrypto extension. The client ID and client secret are used to produce a SHA256 HMAC, which is stored in the configuration file.

The client authenticates its requests to the server using HTTP basic authentication, the server checks the HMAC of the client ID and client secret match the HMAC stored in the configuration file.

See SQL functions in [setup.sql](../../sso/postgres/setup.sql).

-   `sso._secret_generate`
-   `sso._secret_hash`
-   `sso._secret_check`

See Rust functions in [postgres/mod.rs](../../sso/postgres/mod.rs).

-   `Postgres::secret_generate`
-   `Postgres::secret_hash`
-   `Postgres::secret_check`

## User Passwords

-   [OWASP - Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
-   [OWASP - Forgot Password Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html)

User passwords are stored as [bcrypt](https://en.wikipedia.org/wiki/Bcrypt) hashes using the pgcrypto extension with a work factor of 12. User password hashes may be stored in the configuration file and in the database.

Password length is validated by the server with a minimum of 8 and maximum of 64.

HTML client interface includes `zxcvbn` password strength check, this is only run by the user's browser.

Plaintext passwords are not logged or audited by the server, and the code attempts to keep them in memory for as little time as required. But passwords are sent in plaintext to the database as a query parameter for hashing/checking, which could be a problem depending on how database logging is configured.

There is currently no method for upgrading legacy hashes.

See SQL functions in [setup.sql](../../sso/postgres/setup.sql).

-   `sso._password_hash`
-   `sso._password_check`

See Rust functions in [postgres/mod.rs](../../sso/postgres/mod.rs).

-   `Postgres::password_hash`
-   `Postgres::user_password_check`

<!-- todo: Use peppering/pre-hashing? -->
<!-- todo: Legacy hash upgrade mechanism -->

<!-- todo: Urls are https or localhost, how to manage docker domains? -->

## Sessions

-   [OWASP - Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)

## Tokens

User access and refresh tokens are 256 bit random keys generated using the pgcrypto extension. The token ID and access or refresh secret produce a SHA256 HMAC, which is AES encrypted with the client secret.

User access and refresh tokens can be used with the OAuth2 introspection endpoint, the server decrypts the token using the requesting clients secret key, and then checks the token ID and secret HMAC were produced using the token access or refresh secret.

This method is different to client secret checks which is inconsistent, however these should probably be some kind of JWT instead for OIDC support.

-   [OWASP - JSON Web Token Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)

See SQL functions in [setup.sql](../../sso/postgres/setup.sql).

-   `sso._secret_generate`
-   `sso._secret_hash`
-   `sso._secret_encrypt`
-   `sso._secret_decrypt`
-   `sso._secret_check`

## API Keys

-   [OWASP - REST Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/REST_Security_Cheat_Sheet.html)

API keys are 256 bit random keys generated using the pgcrypto extension. The key ID and key secret produce a SHA256 HMAC, which is stored in the database.

User API keys can be verified with the API key verification endpoint, the server checks the HMAC of the key ID and key secret match the HMAC stored in the database.

See SQL functions in [setup.sql](../../sso/postgres/setup.sql).

-   `sso._secret_generate`
-   `sso._secret_hash`
-   `sso._secret_check`

See Rust functions in [postgres/mod.rs](../../sso/postgres/mod.rs).

-   `Postgres::secret_generate`
-   `Postgres::secret_hash`
-   `Postgres::secret_check`
