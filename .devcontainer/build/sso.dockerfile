FROM sso-build/cache-rust:latest as build

# depend: docker pull debian:10.7
FROM debian:10.7
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

# Copy wait-for-it script
COPY .devcontainer/build/scripts/wait-for-it.sh /wait-for-it.sh
RUN chmod +x /wait-for-it.sh

RUN mkdir -p /config
WORKDIR /config
ENTRYPOINT ["sso_server"]
CMD ["--config", "sso"]
