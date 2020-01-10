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

If successful `sso-grpc-server` is now available at `localhost:7042`, and `sso-openapi-server` is available at `localhost:8042`.

```bash
curl localhost:7043/ping
# Pong
curl localhost:8042/ping
# {"code":5}
```
