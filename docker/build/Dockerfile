FROM debian:10.2

ENV DEBIAN_FRONTEND="noninteractive"

# Install dependencies.
RUN apt-get update; \
    apt-get install -y --no-install-recommends \
        wget unzip ca-certificates build-essential libpq-dev libssl-dev pkg-config git; \
    rm -rf /var/lib/apt/lists/*

# Rust environment.
ENV RUSTUP_HOME="/usr/local/rustup" \
    CARGO_HOME="/usr/local/cargo" \
    PATH="/usr/local/cargo/bin:$PATH" \
    RUST_VERSION="1.39.0" \
    RUSTUP_URL="https://static.rust-lang.org/rustup/archive/1.20.2/x86_64-unknown-linux-gnu/rustup-init"

# Go environment.
ENV PATH="/usr/local/go/bin:$PATH" \
    GOLANG_URL="https://golang.org/dl/go1.13.5.linux-amd64.tar.gz" \
    PROTOC_URL="https://github.com/protocolbuffers/protobuf/releases/download/v3.11.1/protoc-3.11.1-linux-x86_64.zip"

# Pandoc environment.
ENV PANDOC_URL="https://github.com/jgm/pandoc/releases/download/2.9/pandoc-2.9-1-amd64.deb"

# Install Rust toolchain.
# <https://github.com/rust-lang/docker-rust>
RUN wget -q "$RUSTUP_URL"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile default --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME;

# Install Rust tools.
RUN cargo install --force cargo-make; \
    cargo install --force diesel_cli --no-default-features --features "postgres"; \
    cargo install --force cargo-audit;

# Install Go toolchain.
# <https://github.com/docker-library/golang>
RUN wget -O go.tgz -q "$GOLANG_URL"; \
    tar -C /usr/local -xzf go.tgz; \
    rm go.tgz; \
    wget -O protoc.zip -q "$PROTOC_URL"; \
    unzip -o protoc.zip -d /usr/local bin/protoc; \
    unzip -o protoc.zip -d /usr/local 'include/*'; \
    rm protoc.zip;

# Install Go tools.
RUN go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway; \
    go get -u github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger; \
    go get -u github.com/golang/protobuf/protoc-gen-go;

# Install Pandoc.
# <https://pandoc.org/installing.html>
RUN wget -O pandoc.deb -q "$PANDOC_URL"; \
    dpkg -i pandoc.deb; \
    rm pandoc.deb;

VOLUME ["/sso"]
WORKDIR /sso

COPY ./versions.sh /versions.sh
RUN chmod +x /versions.sh
CMD ["/versions.sh"]
