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

If successful `sso-grpc-server` is now available at `localhost:7000`, and `sso-openapi-server` is available at `localhost:8000`.

```bash
curl -v localhost:8000/ping
```
