# Guides

## Quickstart

Install [Docker][docker] and [Docker Compose][docker-compose].

- [Docker Desktop (Windows)][docker-desktop-install]
- [Docker Engine (Ubuntu)][docker-engine-install]
- [Docker Compose][docker-compose-install]

Build the `sso/build` [Docker][docker] image.

```bash
docker build --tag "sso/build:latest" .
```

Build [Docker Compose][docker-compose] service images.

```bash
docker-compose build
```

Start services.

```bash
docker-compose up
```

If successful `sso-grpc` is now available at `localhost:7042`, and `sso-openapi` is available at `localhost:8042`.

The following guides depend on the `sso-cli` command, it can be found in the `sso_grpc` container with the command.

```bash
docker exec -it sso_sso-grpc_1 /bin/bash
# root:/# sso-cli --help
```
