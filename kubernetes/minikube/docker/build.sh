#!/bin/bash
# ```bash
# sudo chmod +x build.sh
# bash build.sh
# ```
set -e

docker save "sso/grpc:latest" | (eval $(minikube docker-env) && docker load)
docker save "sso/openapi:latest" | (eval $(minikube docker-env) && docker load)

(cd sso-postgres && docker build --tag "sso/postgres-minikube:latest" .)
docker save "sso/postgres-minikube:latest" | (eval $(minikube docker-env) && docker load)
