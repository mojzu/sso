# Guide

## Environment

| Variable                | Description                                     |
| ----------------------- | ----------------------------------------------- |
| DATABASE_URL            | Database connection URL, required.              |
| SERVER_BIND             | Server bind address, required.                  |
| SMTP_HOST               | SMTP configured, required for emails.           |
| SMTP_PORT               | ...                                             |
| SMTP_USER               | ...                                             |
| SMTP_PASSWORD           | ...                                             |
| SENTRY_URL              | Sentry URL for logging integration, optional.   |
| GITHUB_CLIENT_ID        | GitHub OAuth2 support, optional.                |
| GITHUB_CLIENT_SECRET    |                                                 |
| GITHUB_REDIRECT_URL     | `$server_url/v1/auth/provider/github/oauth2`    |
| MICROSOFT_CLIENT_ID     | Microsoft OAuth2 support, optional.             |
| MICROSOFT_CLIENT_SECRET |                                                 |
| MICROSOFT_REDIRECT_URL  | `$server_url/v1/auth/provider/microsoft/oauth2` |

**Linux**

Write `export $NAME="$VALUE"` statements to file `env.sh` and run `source env.sh` to export variables in open terminal.

## Database

To create [PostgreSQL][postgresql] database migrations.

```shell
cargo make postgres-migrations
```

To create [SQLite][sqlite] database migrations.

```shell
cargo make sqlite-migrations
```

## Build

To build libraries and binaries.

```shell
cargo make build
cargo make release
```

To lint source code using [clippy][clippy].

```shell
cargo make lint
```

[clippy]: https://github.com/rust-lang/rust-clippy

To build documentation.

```shell
cargo doc --no-deps --open
```

To build docker image.

```Shell
cd ark_auth
docker-compose build
```

## Test

To run unit tests.

```shell
cargo make test
```

For integration tests, the following environment variables are required.

| Variable | Description     |
| -------- | --------------- |
| TEST_ARK_AUTH_URL | Server URL.     |
| TEST_ARK_AUTH_KEY | Root key value. |
| TEST_ARK_KV_BIN | Binary path. |

To run unit tests and integration tests.

```shell
cargo make test-integration
```
