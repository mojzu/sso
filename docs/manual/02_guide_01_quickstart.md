# Guides

## Quickstart

Build the `sso-build` [Docker][docker] image.

```bash
docker build --tag "sso-build:latest" .
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

```bash
curl localhost:7043/ping
# Pong
curl localhost:8042/ping
# {"error":"NotFound","code":5,"message":"NotFound"}
```

The following guides depend on `sso-cli` and `curl`. `sso-cli` can be found inside the `sso_grpc` container using the command.

```bash
docker exec -it $container_id /bin/bash
```
