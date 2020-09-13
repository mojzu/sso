FROM sso/build-release:latest as build

# depend: docker pull debian:10.5
FROM debian:10.5
ENV DEBIAN_FRONTEND="noninteractive"

# Install dependencies
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=build /build/target/release/sso_cli /usr/local/bin/sso_cli
COPY --from=build /build/target/release/sso_server /usr/local/bin/sso_server
RUN chmod +x /usr/local/bin/sso_cli \
    /usr/local/bin/sso_server

RUN mkdir -p /config
WORKDIR /config
ENTRYPOINT ["sso_server", "--config", "sso"]
