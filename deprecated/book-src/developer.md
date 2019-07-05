# Developer

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
$ export GITHUB_REDIRECT_URL="http://localhost:9000/v1/auth/provider/github/oauth2"
# Microsoft OAuth2 support, optional.
$ export MICROSOFT_CLIENT_ID="..."
$ export MICROSOFT_CLIENT_SECRET="..."
$ export MICROSOFT_REDIRECT_URL="http://localhost:9000/v1/auth/provider/microsoft/oauth2"
```

### Ubuntu

Write export statements to file `env.sh` and run `source env.sh` to export variables in open terminal.

## Notes

Create database and run migrations.

```shell
$ cd ark_auth
$ diesel database reset --migration-dir migrations/postgres
$ export DATABASE_URL="db.sqlite3"
$ diesel database reset --migration-dir migrations/sqlite
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

Run unit and integration tests. Tests require `TEST_URL` and `TEST_KEY` environment variables to set.

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

Build documentation.

```shell
$ cargo doc --no-deps --open
```

Build docker image.

```Shell
$ cd ark_auth
$ docker-compose build
```
