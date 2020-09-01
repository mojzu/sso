FROM sso/build:latest

# Rust
COPY ./Cargo.toml /build/Cargo.toml
COPY ./Cargo.lock /build/Cargo.lock

# Node
COPY ./package.json /build/package.json
COPY ./package-lock.json /build/package-lock.json

# Build dependencies
WORKDIR /build
RUN mkdir .cargo \
    && cargo vendor > .cargo/config \
    && npm install
