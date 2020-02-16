# Developer

A [Docker][docker] image contains the development tools, build it with the command.

```bash
docker build --tag "sso/build:latest" .
```

Create a network for containers.

```bash
docker network create compose
```

Development tools are run with the command.

```bash
docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" sso/build:latest $ARGS
```

Create an alias on Linux for the above with the command.

```bash
alias sso-build='docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" sso/build:latest'
```

Development environment variables are configured in `Dockerfile`.

Services are run using [Docker Compose][docker-compose], start them with the command.

```bash
docker-compose build
docker-compose up
```

Create an alias on Linux to run `sso-build` container with a hostname, this allows you to replace compose services during development without using host networking mode.

```bash
sso-build-host() {
    local host="$1"
    shift 1
    docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" --hostname $host --name $host sso/build:latest "$@"
}
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
sso-build cargo make sso-grpc
sso-build cargo make sso-openapi
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

## Minikube

Create a [Minikube][minikube] instance.

```bash
minikube start --vm-driver=virtualbox
minikube status
minikube stop
minikube delete
minikube dashboard
minikube ip
```

(Re)build all Docker images and load images into Minikube.

```bash
docker build --tag "sso-build:latest" .
docker-compose build --parallel
(cd kubernetes/minikube/docker && bash build.sh)
```
