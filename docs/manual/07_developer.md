# Developer

## Tools

A [Docker][docker] image contains the development tools, build it with the command.

```bash
docker build --tag "sso_build:latest" docker/build
```

Development tools are run with the command.

```bash
docker run --rm -v "$(pwd):/sso" -t sso_build:latest $ARGS
```

Create an alias on Linux for the above with the command.

```bash
alias sso_build='docker run --rm -v "$(pwd):/sso" -t sso_build:latest'
```

Services are run using [Docker Compose][docker-compose], start them with the command.

```bash
docker-compose build
docker-compose up postgres
```

Stop and destroy services with the commands.

```bash
docker-compose stop
docker-compose down
```

Create backup of `sso` database in `postgres` service. This backup will be restored automatically when the `postgres` service is rebuilt.

```bash
docker exec sso_postgres_1 pg_dump -U guest --format=custom sso > docker/postgres/pgdump/sso.pgdump
```

## Manual

This manual is written in [Markdown][pandoc-markdown] and converted into other formats using [Pandoc][pandoc].

To build the HTML manual.

```bash
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
| PASSWORD_PWNED_ENABLED  | Enable [Pwned Passwords][pwned-passwords] integration. |
| GITHUB_CLIENT_ID        | GitHub OAuth2 provider, optional.                      |
| GITHUB_CLIENT_SECRET    | ...                                                    |
| MICROSOFT_CLIENT_ID     | Microsoft OAuth2 provider, optional.                   |
| MICROSOFT_CLIENT_SECRET | ...                                                    |

### Ubuntu

Write `export $NAME="$VALUE"` statements to file `.env` and run `source .env` to export variables in open terminal. See `sso/.env` for example.

## Database

To reset and create new [PostgreSQL][postgresql] database migrations for [Diesel][diesel]. These commands assume docker container is running.

```bash
cargo make postgres-reset
cargo make postgres-migration $migration_name
```

To reset and create new [SQLite][sqlite] database migrations for [Diesel][diesel].

```bash
cargo make sqlite-reset
cargo make sqlite-migration $migration_name
```

## Build

To build libraries and binaries.

```bash
cargo make build
cargo make release
cargo make release-flow
cargo make install
```

To lint source code using [clippy][clippy].

```bash
cargo make clippy
```

To build and open documentation.

```bash
cargo make doc
```

To build docker image.

```bash
cargo make docker-build
```

To audit crate dependencies.

```bash
cargo make audit
```

[To publish crate(s)][cargo-publishing].

## Test

To run unit tests.

```bash
cargo make test
```

For integration tests, the following environment variables are required. A server hosting the API to be tested must be available at `SERVER_URL` address.

| Variable     | Description     |
| ------------ | --------------- |
| TEST_SSO_URL | Server URL.     |
| TEST_SSO_KEY | Root key value. |

To run integration tests.

```bash
cargo make test-integration
```
