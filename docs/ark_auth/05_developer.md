# Developer

## Tools

The ark_auth crate depends on [PostgreSQL](https://www.postgresql.org/) and [SQLite](https://www.sqlite.org/index.html) libraries.

```shell
# Install on Ubuntu.
sudo apt install libpq-dev libsqlite3-dev libssl-dev libfuse-dev pkg-config
```

Install [Rust](https://www.rust-lang.org/) using [rustup](https://rustup.rs/).

To update the toolchain.

```shell
rustup self update
rustup update
```

The ark_auth crate depends on [Cargo Make](https://github.com/sagiegurari/cargo-make) and [Diesel](http://diesel.rs/), install them with Cargo.

```shell
cargo install --force cargo-make
cargo install --force diesel_cli --no-default-features --features "postgres sqlite"
```

To update crate dependencies.

```shell
cargo update
```

[Docker](https://docs.docker.com/install/) and [Docker Compose](https://docs.docker.com/compose/) are used for development, install them using the linked documentation.

To start containers defined in `docker-compose.yml`.

```shell
docker-compose up
```

To stop containers.

```shell
docker-compose down
```

## Manual

This manual is written in [Markdown](https://pandoc.org/MANUAL.html#pandocs-markdown) and converted into other formats using [Pandoc](https://pandoc.org/).

To build the HTML manual.

```shell
cargo make manual
```

## Environment

| Variable                | Description                                     |
| ----------------------- | ----------------------------------------------- |
| SENTRY_URL              | Sentry URL for logging integration, optional.   |
| DATABASE_URL            | Database connection URL, required.              |
| DATABASE_CONNECTIONS    | Database connections, optional.                 |
| SERVER_HOSTNAME         | Server hostname, optional.                      |
| SERVER_BIND             | Server bind address, required.                  |
| SERVER_TLS_CRT_PEM      | Server TLS certificate files, optional.         |
| SERVER_TLS_KEY_PEM      |                                                 |
| SERVER_TLS_CLIENT_PEM   | Server mutual TLS authentication, optional.     |
| SMTP_HOST               | SMTP server, optional.                          |
| SMTP_PORT               | ...                                             |
| SMTP_USER               | ...                                             |
| SMTP_PASSWORD           | ...                                             |
| GITHUB_CLIENT_ID        | GitHub OAuth2 provider, optional.               |
| GITHUB_CLIENT_SECRET    |                                                 |
| GITHUB_REDIRECT_URL     | `$server_url/v1/auth/provider/github/oauth2`    |
| MICROSOFT_CLIENT_ID     | Microsoft OAuth2 provider, optional.            |
| MICROSOFT_CLIENT_SECRET |                                                 |
| MICROSOFT_REDIRECT_URL  | `$server_url/v1/auth/provider/microsoft/oauth2` |

**Linux**

Write `export $NAME="$VALUE"` statements to file `.env` and run `source .env` to export variables in open terminal. See `ark_auth/.env` for example.

## Database

To create [PostgreSQL](https://www.postgresql.org/) database migrations.

```shell
cargo make postgres-migrations
```

To create [SQLite](https://www.sqlite.org/index.html) database migrations.

```shell
cargo make sqlite-migrations
```

## Test

To run unit tests.

```shell
cargo make test
```

For integration tests, the following environment variables are required.

| Variable          | Description     |
| ----------------- | --------------- |
| TEST_ARK_AUTH_URL | Server URL.     |
| TEST_ARK_AUTH_KEY | Root key value. |

To run integration tests.

```shell
cargo make test-integration
```

## Build

To build libraries and binaries.

```shell
cargo make build
cargo make release
```

To lint source code using [clippy](https://github.com/rust-lang/rust-clippy).

```shell
cargo make lint
```

To build documentation.

```shell
cargo doc --no-deps --open
```

To build docker image.

```Shell
docker-compose build
```
