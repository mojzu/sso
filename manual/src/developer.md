# Developer

## Tools

- [Rust](https://www.rust-lang.org/)
- [Diesel](http://diesel.rs/)
- [mdBook](https://github.com/rust-lang-nursery/mdBook)
- [Docker](https://docs.docker.com/install/)
- [Docker Compose](https://docs.docker.com/compose/)

```shell
# Rust installation, updates and utilities.
$ curl https://sh.rustup.rs -sSf | sh
$ rustup self update && rustup update && cargo update
$ rustup component add rustfmt
$ rustup component add clippy
# Diesel CLI, mdBook installation.
$ cargo install diesel_cli --no-default-features --features "postgres sqlite"
$ cargo install mdbook
# Docker, Docker Compose installation and start containers.
$ docker-compose up
```

### Ubuntu

```shell
$ sudo apt install libpq-dev libsqlite3-dev libssl-dev pkg-config
```

## Environment

Replace `...` as required.

```shell
# Database connection URL, required.
$ export DATABASE_URL="postgres://ark_auth:guest@localhost:5432/ark_auth"
# Server bind, required.
$ export SERVER_BIND="localhost:9000"
# SMTP configured, required for reset password emails.
$ export SMTP_HOST="..."
$ export SMTP_PORT="..."
$ export SMTP_USER="..."
$ export SMTP_PASSWORD="..."
# Sentry URL for logging integration, optional.
$ export SENTRY_URL="..."
# GitHub OAuth2 support, optional.
$ export GITHUB_CLIENT_ID="..."
$ export GITHUB_CLIENT_SECRET="..."
$ export GITHUB_REDIRECT_URL="http://localhost:9000/v1/auth/oauth2/github"
# Microsoft OAuth2 support, optional.
$ export MICROSOFT_CLIENT_ID="..."
$ export MICROSOFT_CLIENT_SECRET="..."
$ export MICROSOFT_REDIRECT_URL="http://localhost:9000/v1/auth/oauth2/microsoft"
```

### Ubuntu

Write export statements to file `env.sh` and run `source env.sh` to export variables in open terminal.

## Notes

Create database and run migrations.

```shell
$ diesel database reset --migration-dir ark_auth/migrations/postgres
$ diesel database reset --migration-dir ark_auth/migrations/sqlite
```

Build application, run commands (see help).

```shell
$ cargo build [--release]
$ cargo run help
```

Format source code and run clippy.

```shell
$ cargo fmt && cargo clippy
```

Run unit and integration tests. Integration tests are run from manual markdown files using `skeptic`, which require `TEST_URL` and `TEST_KEY` environment variables to set. In case of dependency resolution problems while compiling, run `cargo clean`.

```shell
$ export TEST_URL="..."
$ export TEST_KEY="..."
$ diesel database reset --migration-dir ark_auth/migrations/postgres
$ cargo test [--test $test_name]
```

Serve and build manual.

```shell
$ cd manual
$ mdbook serve
$ rm -rf ../docs && mdbook build
```

Build docker image.

```Shell
$ cd ark_auth
$ docker-compose build
```
