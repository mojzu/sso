# Developer

## Requirements

-   [Rust](https://www.rust-lang.org/)
-   [Diesel](http://diesel.rs/)

```Shell
# Rust installation, updates and utilities.
$ curl https://sh.rustup.rs -sSf | sh
$ rustup self update && rustup update && cargo update
$ rustup component add rustfmt
$ rustup component add clippy
# Diesel CLI installation.
$ cargo install diesel_cli --no-default-features --features "postgres sqlite"
```

Create a database user with permission to create databases.

```Shell
$ sudo -i -u postgres
(postgres) $ psql -c "CREATE USER $db_user WITH PASSWORD '$db_password';";
(postgres) $ psql -c "ALTER USER $db_user CREATEDB;";
```

Export environment variables.

```Shell
# Database connection URL, required.
$ export DATABASE_URL="postgres://$db_user:$db_password@$db_host:$db_port/ark_auth"
# Server address, required.
$ export SERVER_ADDR="$server_host:$server_port"
# Sentry URL for logging integration, optional.
$ export SENTRY_URL="..."
# GitHub OAuth support, optional.
$ export GITHUB_CLIENT_ID="..."
$ export GITHUB_CLIENT_SECRET="..."
$ export GITHUB_REDIRECT_URL="$server_url/v1/auth/oauth/github"
# Microsoft OAuth support, optional.
$ export MICROSOFT_CLIENT_ID="..."
$ export MICROSOFT_CLIENT_SECRET="..."
$ export MICROSOFT_REDIRECT_URL="$server_url/v1/auth/oauth/microsoft"
```

Create database and run migrations.

```Shell
$ diesel database reset
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

Build manual.

```Shell
$ mdbook serve
```

Run unit and integration tests.

```Shell
$ diesel database reset && cargo test
```
