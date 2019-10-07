# Developer

## Tools

The `sso` crate depends on [PostgreSQL](postgresql) and [SQLite](sqlite) libraries.

```shell
# Install on Ubuntu.
sudo apt install libpq-dev libsqlite3-dev libssl-dev libfuse-dev pkg-config
```

Install [Rust](rust) using [rustup](rustup).

To update the toolchain.

```shell
rustup self update
rustup update
```

The `sso` crate depends on [cargo make](cargo-make), [Diesel](diesel) and [cargo audit](cargo-audit), install them with Cargo.

```shell
cargo install --force cargo-make
cargo install --force diesel_cli --no-default-features --features "postgres sqlite"
cargo install --force cargo-audit
```

To update crate dependencies.

```shell
cargo update
```

[Docker](docker) and [Docker Compose](docker-compose) are used for development, install them using the linked documentation.

To start containers defined in `docker/docker-compose.yml`.

```shell
cargo make docker-up
```

To stop and destroy containers.

```shell
cargo make docker-stop
cargo make docker-down
```

## Manual

This manual is written in [Markdown](pandoc-markdown) and converted into other formats using [Pandoc](pandoc).

To build the HTML manual.

```shell
cargo make manual
```

## Environment

| Variable                | Description                                            |
| ----------------------- | ------------------------------------------------------ |
| SENTRY_URL              | Sentry URL for logging integration, optional.          |
| DATABASE_URL            | Database connection URL, required.                     |
| DATABASE_CONNECTIONS    | Database connections, optional.                        |
| SERVER_HOSTNAME         | Server hostname, optional.                             |
| SERVER_BIND             | Server bind address, required.                         |
| SERVER_TLS_CRT_PEM      | Server TLS certificate files, optional.                |
| SERVER_TLS_KEY_PEM      |                                                        |
| SERVER_TLS_CLIENT_PEM   | Server mutual TLS authentication, optional.            |
| SMTP_HOST               | SMTP server, optional.                                 |
| SMTP_PORT               | ...                                                    |
| SMTP_USER               | ...                                                    |
| SMTP_PASSWORD           | ...                                                    |
| PASSWORD_PWNED_ENABLED  | Enable [Pwned Passwords](pwned-passwords) integration. |
| GITHUB_CLIENT_ID        | GitHub OAuth2 provider, optional.                      |
| GITHUB_CLIENT_SECRET    | ...                                                    |
| MICROSOFT_CLIENT_ID     | Microsoft OAuth2 provider, optional.                   |
| MICROSOFT_CLIENT_SECRET | ...                                                    |

**Ubuntu**

Write `export $NAME="$VALUE"` statements to file `.env` and run `source .env` to export variables in open terminal. See `sso/.env` for example.

## Database

To reset and create new [PostgreSQL](postgresql) database migrations for [Diesel](diesel). These commands assume docker container is running.

```shell
cargo make postgres-reset
cargo make postgres-migration $migration_name
```

To reset and create new [SQLite](sqlite) database migrations for [Diesel](diesel).

```shell
cargo make sqlite-reset
cargo make sqlite-migration $migration_name
```

## Build

To build libraries and binaries.

```shell
cargo make build
cargo make release
cargo make release-flow
cargo install --force --path sso
```

To lint source code using [clippy](clippy).

```shell
cargo make clippy
```

To build and open documentation.

```shell
cargo make doc
```

To build docker image.

```Shell
cargo make docker-build
```

To audit crate dependencies.

```Shell
cargo make audit
```

[To publish crate(s)](cargo-publishing).

## Test

To run unit tests.

```shell
cargo make test
```

For integration tests, the following environment variables are required. A server hosting the API to be tested must be available at `SERVER_URL` address.

| Variable     | Description     |
| ------------ | --------------- |
| TEST_SSO_URL | Server URL.     |
| TEST_SSO_KEY | Root key value. |

To run integration tests.

```shell
cargo make test-integration
```
