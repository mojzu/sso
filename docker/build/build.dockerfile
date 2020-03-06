FROM debian:10.3
ENV DEBIAN_FRONTEND="noninteractive"

# User ID argument to match host.
ARG UID

# Install dependencies.
RUN apt-get update && \
    apt-get install -y -q --no-install-recommends \
    wget unzip ca-certificates build-essential libpq-dev libssl-dev pkg-config git procps \
    gnupg2 apt-transport-https openssh-client curl && \
    rm -rf /var/lib/apt/lists/*;

# Create user to match host.
RUN useradd --uid $UID --shell /bin/bash --create-home build;

# Environment.
ENV HOME="/root"

# Rust environment.
ENV RUSTUP_HOME="/usr/local/rustup" \
    CARGO_HOME="/usr/local/cargo" \
    PATH="/usr/local/cargo/bin:$PATH" \
    RUST_VERSION="1.41.1" \
    RUSTUP_VERSION_URL="https://static.rust-lang.org/rustup/archive/1.21.1/x86_64-unknown-linux-gnu/rustup-init"

# Install Rust toolchain.
# <https://github.com/rust-lang/docker-rust>
RUN wget -q "$RUSTUP_VERSION_URL" && \
    chmod +x rustup-init && \
    ./rustup-init -y --no-modify-path --profile default --default-toolchain $RUST_VERSION && \
    rm rustup-init && \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME && \
    chmod 777 -R $HOME;

# Install Rust tools.
RUN cargo install --force cargo-make --version "~0.28" && \
    cargo install --force diesel_cli --version "~1.4" --no-default-features --features "postgres" && \
    cargo install --force cargo-audit --version "~0.11";

# Go environment.
ENV PATH="/usr/local/go/bin:/root/go/bin:$PATH" \
    GOLANG_VERSION_URL="https://dl.google.com/go/go1.14.linux-amd64.tar.gz" \
    PROTOC_VERSION_URL="https://github.com/protocolbuffers/protobuf/releases/download/v3.11.4/protoc-3.11.4-linux-x86_64.zip"

# Install Go toolchain.
# <https://github.com/docker-library/golang>
RUN wget -O go.tgz -q "$GOLANG_VERSION_URL" && \
    tar -C /usr/local -xzf go.tgz && \
    rm go.tgz && \
    wget -O protoc.zip -q "$PROTOC_VERSION_URL" && \
    unzip -o protoc.zip -d /usr/local bin/protoc && \
    unzip -o protoc.zip -d /usr/local 'include/*' && \
    chmod -R 777 /usr/local/bin/protoc && \
    chmod -R 777 /usr/local/include/google && \
    rm protoc.zip;

# Install Go tools.
# <https://github.com/grpc-ecosystem/grpc-gateway>
# <https://grpc-ecosystem.github.io/grpc-gateway/>
RUN go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway && \
    go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger && \
    go get -u github.com/golang/protobuf/protoc-gen-go && \
    go get -u google.golang.org/grpc;

# Dart environment.
ENV DART_VERSION="2.7.1" \
    DART_SDK="/usr/lib/dart" \
    PATH="/usr/lib/dart/bin:/root/.pub-cache/bin:$PATH" \
    PROTOC_PLUGIN_VERSION="19.0.0+1"

# Install Dart tools.
# <https://github.com/dart-lang/dart_docker>
# <https://github.com/dart-lang/protobuf>
RUN curl https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - && \
    curl https://storage.googleapis.com/download.dartlang.org/linux/debian/dart_stable.list > /etc/apt/sources.list.d/dart_stable.list && \
    curl https://storage.googleapis.com/download.dartlang.org/linux/debian/dart_unstable.list > /etc/apt/sources.list.d/dart_unstable.list && \
    apt-get update && \
    apt-get install dart=$DART_VERSION-1 && \
    rm -rf /var/lib/apt/lists/* && \
    pub global activate protoc_plugin $PROTOC_PLUGIN_VERSION;

# Set cargo cache directory in volume.
# This prevents having to download dependencies in development builds.
ENV CARGO_HOME="/build/.cargo"

# Copy CA certificate files.
ADD ./docker/build/cert /cert
RUN chmod +r -R /cert

# Set working directory.
WORKDIR /build

# Print installed versions script default command.
ADD ./docker/build/versions.sh /versions.sh
RUN chmod +x /versions.sh
CMD ["/versions.sh"]

# -----------------------------
# START Development Environment
# -----------------------------
# This file is checked into Git and must not contain secrets!
ENV RUST_BACKTRACE="1" \
    RUST_LOG="info,sso=debug"

# # Sentry DSN for logging integration.
# ENV SSO_SENTRY_DSN=""
ENV SSO_LOG_PRETTY="true"
# Postgres connection.
ENV SSO_POSTGRES_URL="postgres://guest:guest@postgres:5432/sso" \
    SSO_POSTGRES_CONNECTIONS="10"
# # Server TLS.
# ENV SSO_TLS_CERT_PEM="" \
#     SSO_TLS_KEY_PEM="" \
#     SSO_TLS_CLIENT_PEM=""
# # SMTP server transport.
# ENV SSO_SMTP_HOST="" \
#     SSO_SMTP_PORT="" \
#     SSO_SMTP_USER="" \
#     SSO_SMTP_PASSWORD=""
# SMTP file transport.
ENV SSO_SMTP_FILE="./tmp"
# Pwned Passwords integration.
ENV SSO_PWNED_PASSWORDS="true"
# Traefik forward authentication integration.
ENV SSO_TRAEFIK="false"
# # Github OAuth2 support.
# ENV SSO_GITHUB_CLIENT_ID="" \
#     SSO_GITHUB_CLIENT_SECRET=""
# # Microsoft OAuth2 support.
# ENV SSO_MICROSOFT_CLIENT_ID="" \
#     SSO_MICROSOFT_CLIENT_SECRET=""
# gRPC server URL.
ENV SSO_GRPC_URL="sso-grpc:7042"
# Integration test variables.
ENV SSO_TEST_URL="http://traefik:80" \
    SSO_TEST_KEY="UAMK24IW72UTDXZUY45MIPBPDRPIARQR6M"
# Integration test TLS variables.
# ENV SSO_TEST_TLS_DOMAIN=""
# ENV SSO_TEST_TLS_CA_CERT="/cert/root_ca.crt"
# ENV SSO_TEST_TLS_CLIENT_CERT="" \
#     SSO_TEST_TLS_CLIENT_KEY=""
