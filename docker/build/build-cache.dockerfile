FROM sso/build:v1

# Create empty project.
RUN USER=root cargo new --lib /build/sso;

# Copy manifest files.
COPY ./docker/build/Cargo.toml /build/Cargo.toml
COPY ./docker/build/Cargo.lock /build/Cargo.lock
COPY ./sso/Cargo.toml /build/sso/Cargo.toml

# Build dependencies.
RUN cargo build --release --lib;
