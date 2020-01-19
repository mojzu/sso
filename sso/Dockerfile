FROM sso-build:latest as build

# Copy files and build binary.
ADD . /sso/sso
RUN cargo make release;

FROM debian:10.2
ENV DEBIAN_FRONTEND="noninteractive"

# gRPC server port.
EXPOSE 7042

# HTTP server port.
EXPOSE 7043

# Install dependencies.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates libpq-dev \
    && rm -rf /var/lib/apt/lists/*;

# Copy binaries.
COPY --from=build /sso/target/release/sso-grpc /usr/local/bin/sso-grpc
COPY --from=build /sso/target/release/sso-cli /usr/local/bin/sso-cli
RUN chmod +x /usr/local/bin/sso-grpc /usr/local/bin/sso-cli

ENTRYPOINT ["sso-grpc"]
