# Developer

A [Docker][docker] image contains the development tools, build it with the command.

```bash
docker build --tag "sso-build:latest" .
```

Development tools are run with the command.

```bash
docker run --rm --user $(id -u):$(id -g) --network host -v "$(pwd):/sso" -t sso-build:latest $ARGS
```

Create an alias on Linux for the above with the command.

```bash
alias sso-build='docker run -it --rm --init --user $(id -u):$(id -g) --network host -v "$(pwd):/sso" -t sso-build:latest'
```

Environment variables are configured in `Dockerfile`.

Services are run using [Docker Compose][docker-compose], start them with the command.

```bash
docker-compose build
docker-compose up
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

Reset database and create new [PostgreSQL][postgresql] database migrations with [Diesel][diesel]. These commands expect `postgres` service is running.

```bash
sso-build cargo make postgres-reset
sso-build cargo make postgres-migration $migration_name
```

Check source code using [clippy][clippy].

```bash
sso-build cargo make clippy
```

Audit crate dependencies.

```bash
sso-build cargo make audit
```

Build libraries and binaries.

```bash
sso-build cargo make build
sso-build cargo make release
```

Run development binaries.

```bash
sso-build cargo make sso-cli -- $ARGS
sso-build cargo make sso-grpc-server
sso-build cargo make sso-openapi-server
```

Run unit tests.

```bash
sso-build cargo make test
```

Run integration tests. This expects `sso_grpc` service is running.

```bash
sso-build cargo make test-integration
```

Compile [Protocol Buffers][protocol-buffers] for [OpenAPI][openapi] gateway server.

```bash
sso-build cargo make openapi-protoc
```

Build [OpenAPI][openapi] gateway server.

```bash
sso-build cargo make openapi-bin
```

This manual is written in [Markdown][pandoc-markdown] and converted into other formats using [Pandoc][pandoc]. Build the HTML manual.

```bash
sso-build cargo make manual
```

Build crate documentation.

```bash
sso-build cargo make doc
```

Install crate locally.

```bash
cargo install --force --path sso
```

[To publish crate(s)][cargo-publishing].
