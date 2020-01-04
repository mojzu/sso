FROM sso-build:latest as build

# Copy files and build binary.
ADD . /sso/sso
RUN cargo make release;

FROM debian:10.2
ENV DEBIAN_FRONTEND="noninteractive"

# Install dependencies.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates libpq-dev \
    && rm -rf /var/lib/apt/lists/*;

# Copy and start server binary.
COPY --from=build /sso/target/release/sso-grpc-server .
CMD ["./sso-grpc-server"]
