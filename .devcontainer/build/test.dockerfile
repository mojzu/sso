FROM sso-build/cache-node:latest as build

# depend: docker pull node:14.13-buster
FROM node:14.13-buster

# Install dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy files
WORKDIR /opt
COPY --from=build /build/package.json /opt/package.json
COPY --from=build /build/package-lock.json /opt/package-lock.json
COPY --from=build /build/node_modules /opt/node_modules
COPY --from=build /build/sso_test /opt/sso_test

# Copy wait-for-it script
COPY .devcontainer/build/scripts/wait-for-it.sh /wait-for-it.sh
RUN chmod +x /wait-for-it.sh
