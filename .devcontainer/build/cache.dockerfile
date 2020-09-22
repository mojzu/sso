FROM sso/build:latest

WORKDIR /build

# Rust
COPY ./Cargo.toml /build/Cargo.toml
COPY ./Cargo.lock /build/Cargo.lock
COPY ./sso_client/Cargo.toml /build/sso_client/Cargo.toml
RUN mkdir .cargo \
    && cargo vendor > .cargo/config

# Node
COPY ./package.json /build/package.json
COPY ./package-lock.json /build/package-lock.json
RUN npm install
