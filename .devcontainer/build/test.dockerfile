FROM sso/build-release:latest as build

FROM node:14.9.0-buster

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt
COPY --from=build /build/package.json /opt/package.json
COPY --from=build /build/package-lock.json /opt/package-lock.json
COPY --from=build /build/node_modules /opt/node_modules
COPY --from=build /build/sso_test /opt/sso_test
