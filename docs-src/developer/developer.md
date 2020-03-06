# Developer

[Docker][docker] images contain the development tools, build them with the command.

```bash
source docker/alias.sh
sso-build-build
```

Create a network for containers.

```bash
docker network create compose
```

Development tools are run with the command.

```bash
docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" sso/build:v1 $ARGS
```

Create an alias on Linux for the above with the command.

```bash
alias sso-build='docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" sso/build:v1'
```

Development environment variables are configured in `docker/build/build.dockerfile`.

Services are run using [Docker Compose][docker-compose], start them with the command.

```bash
sso build
sso up
```

Create an alias on Linux to run `sso-build` container with a hostname, this allows you to replace compose services during development without using host networking mode.

```bash
sso-build-host() {
    local host="$1"
    shift 1
    docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" --hostname $host --name $host sso/build:v1 "$@"
}
```

To use the `sso-cli` command you can access the `sso-grpc` container.

```bash
docker exec -it sso_sso-grpc_1 /bin/bash
sso-cli --help
sso-cli create-root-key root
sso-cli create-service-with-key test test.localhost --allow-register true --local-url test.localhost/auth/provider/local
```

Stop and destroy services with the commands.

```bash
sso stop
sso down
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

Run integration tests. This expects `sso-grpc` service is running and `SSO_TEST_URL` and `SSO_TEST_KEY` environment variables are defined. Where URL is the address of the gRPC server and key is a root key value returned by `sso-cli`.

```bash
sso-build cargo make test-integration
for i in {1..50}; do sso-build cargo make test-integration; done
```

Compile [Protocol Buffers][protocol-buffers] for [OpenAPI][openapi] gateway server.

```bash
sso-build cargo make protoc
```

Build [OpenAPI][openapi] gateway server.

```bash
sso-build cargo make openapi-bin
```

This manual is written in [Markdown][pandoc-markdown] and build into a static website using [mkdocs][mkdocs].

```bash
sso-mkdocs serve
sso-mkdocs build
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
docker build --tag "sso-build:" .
docker-compose build --parallel
(cd kubernetes/minikube/docker && bash build.sh)
```

## Smallstep CA

Create CA root certificate and certificate and key for server for testing, overwrite files in `docker/build/cert`.

```bash
mkdir -p volumes/ca
docker run --rm --user $(id -u):$(id -g) --network host -v "$(pwd)/volumes/ca:/home/step" -it smallstep/step-ca step ca init
# STDIN: Localhost, localhost, :443, localhost, guest
echo "guest" > volumes/ca/secrets/password
docker run --rm --user $(id -u):$(id -g) --network host -v "$(pwd)/volumes/ca:/home/step" -it smallstep/step-ca /bin/bash
step ca certificate --offline --not-after 999h --san traefik sso.localhost sso.crt sso.key
step certificate inspect sso.crt
# STDIN: guest
```

Modify `volumes/ca/config/ca.json`.

```json
{
  "authority": {
    "claims": {
      "maxTLSCertDuration": "999h",
      "defaultTLSCertDuration": "999h"
    }
  }
}
```

[docker]: https://docs.docker.com/install/
[docker-compose]: https://docs.docker.com/compose/
[diesel]: http://diesel.rs/
[postgresql]: https://www.postgresql.org/
[clippy]: https://github.com/rust-lang/rust-clippy
[protocol-buffers]: https://developers.google.com/protocol-buffers/
[openapi]: https://swagger.io/docs/specification/about/
[pandoc-markdown]: https://pandoc.org/MANUAL.html#pandocs-markdown
[mkdocs]: https://www.mkdocs.org/
[cargo-publishing]: https://doc.rust-lang.org/cargo/reference/publishing.html
[minikube]: https://kubernetes.io/docs/tasks/tools/install-minikube/
