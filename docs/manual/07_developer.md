# Developer

A [Docker][docker] image contains the development tools, build it with the command.

```bash
docker build --tag "sso_build:latest" .
```

Development tools are run with the command.

```bash
docker run --rm --network host -v "$(pwd):/sso" -t sso_build:latest $ARGS
```

Create an alias on Linux for the above with the command.

```bash
alias sso_build='docker run --rm --network host -v "$(pwd):/sso" -t sso_build:latest'
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

Reset database and create new [PostgreSQL][postgresql] database migrations with [Diesel][diesel]. These commands expect `postgres` service is running.

```bash
sso_build cargo make postgres-reset
sso_build cargo make postgres-migration $migration_name
```

Check source code using [clippy][clippy].

```bash
sso_build cargo make clippy
```

Audit crate dependencies.

```bash
sso_build cargo make audit
```

Build libraries and binaries.

```bash
sso_build cargo make build
sso_build cargo make release
```

Run unit tests.

```bash
sso_build cargo make test
```

Run integration tests. This expects `sso_grpc` service is running.

```bash
sso_build cargo make test-integration
```

Compile [Protocol Buffers][protocol-buffers] for [OpenAPI][openapi] gateway server.

```bash
sso_build cargo make openapi-protoc
```

Build [OpenAPI][openapi] gateway server.

```bash
sso_build cargo make openapi-bin
```

This manual is written in [Markdown][pandoc-markdown] and converted into other formats using [Pandoc][pandoc]. Build the HTML manual.

```bash
sso_build cargo make manual
```

Build crate documentation.

```bash
sso_build cargo make doc
```

Install crate locally.

```bash
sso_build cargo clean
cargo install --force --path sso_grpc
```

[To publish crate(s)][cargo-publishing].

Export environment variables from file. See `sso/.env` for example.

```bash
source .env
```
