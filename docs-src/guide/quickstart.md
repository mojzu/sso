# Quickstart

Install [Docker][docker] and [Docker Compose][docker-compose].

- [Docker Desktop (Windows)][docker-desktop-install]
- [Docker Engine (Ubuntu)][docker-engine-install]
- [Docker Compose][docker-compose-install]

Build [Docker Compose][docker-compose] service images and create network.

```bash
export UID
docker-compose -f docker/build.yml build
docker-compose -f docker/sso.yml build
docker network create compose
```

Start services.

```bash
docker-compose -f docker/sso.yml up
```

If successful `sso-grpc` is now available at `sso.localhost`, and `sso-openapi` is available at `sso.localhost/api`.

The following guides depend on the `sso-cli` command, it can be found in the `sso_grpc` container with the command.

```bash
docker exec -it docker_sso-grpc_1 /bin/bash
# root:/# sso-cli --help
```

Docker services can be destroyed with the command.

```bash
docker-compose -f docker/sso.yml down
```

[docker]: https://docs.docker.com/install/
[docker-compose]: https://docs.docker.com/compose/
[docker-desktop-install]: https://docs.docker.com/docker-for-windows/install/
[docker-engine-install]: https://docs.docker.com/install/linux/docker-ce/ubuntu/
[docker-compose-install]: https://docs.docker.com/compose/install/
