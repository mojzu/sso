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

Build [Docker Compose][docker-compose] service images and create network.

```bash
docker-compose build
docker network create compose
```

Start services.

```bash
docker-compose up
```

If successful `sso-grpc` is now available at `sso.localhost`, and `sso-openapi` is available at `sso.localhost/api`.

The following guides depend on the `sso-cli` command, it can be found in the `sso_grpc` container with the command.

```bash
docker exec -it sso_sso-grpc_1 /bin/bash
# root:/# sso-cli --help
```

Services can be destroyed with the command.

```bash
docker-compose down
```
