FROM sso/build:latest as build-1

# Rust build.
RUN cargo make release;

FROM sso/build:latest as build-2

# Go build.
RUN cargo make openapi-bin;

FROM debian:10.2
ENV DEBIAN_FRONTEND="noninteractive"

# gRPC server gRPC port.
EXPOSE 7042

# gRPC server HTTP port.
EXPOSE 7043

# OpenAPI server HTTP port.
EXPOSE 8042

# Install dependencies.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates libpq-dev \
    && rm -rf /var/lib/apt/lists/*;

# Copy binaries.
COPY --from=build-1 /build/target/release/sso-grpc /usr/local/bin/sso-grpc
COPY --from=build-1 /build/target/release/sso-cli /usr/local/bin/sso-cli
COPY --from=build-2 /build/sso_openapi/sso-openapi /usr/local/bin/sso-openapi
RUN chmod +x /usr/local/bin/sso-grpc \
    /usr/local/bin/sso-cli \
    /usr/local/bin/sso-openapi;

# Default entrypoint.
ENTRYPOINT ["sso-grpc"]
