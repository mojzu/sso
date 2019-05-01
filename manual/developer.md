# Developer

## Requirements

-   [Rust](https://www.rust-lang.org/)
-   [Diesel](http://diesel.rs/)
-   [mdBook](https://github.com/rust-lang-nursery/mdBook)
-   [Docker](https://docs.docker.com/install/)
-   [Docker Compose](https://docs.docker.com/compose/)

```Shell
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

```Shell
$ sudo apt install libpq-dev libsqlite3-dev libssl-dev pkg-config
```

## Notes

Export environment variables.

```Shell
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

Create database and run migrations.

```Shell
$ diesel database reset --migration-dir migrations/postgres
```

Build application, initialise database with service, start server.

```Shell
$ cargo build [--release]
$ cargo run init $service_name $service_url
$ cargo run start
```

Format source code and run clippy.

```Shell
$ cargo fmt && cargo clippy
```

Serve and build manual.

```Shell
$ mdbook serve
$ rm -rf docs && mdbook build
```

Run unit and integration tests.

```Shell
$ diesel database reset --migration-dir migrations/postgres && cargo test [--test $test_name]
```
