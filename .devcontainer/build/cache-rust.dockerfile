# depend: docker pull rust:1.49.0-buster
FROM rust:1.49.0-buster

WORKDIR /build

COPY ./Cargo.toml /build/Cargo.toml
COPY ./Cargo.lock /build/Cargo.lock
COPY ./sso_client/Cargo.toml /build/sso_client/Cargo.toml
RUN mkdir .cargo \
    && cargo vendor > .cargo/config

COPY ./sso /build/sso
RUN cargo build --release
