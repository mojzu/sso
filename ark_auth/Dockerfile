# Build stage.
FROM rust:1.34.2-slim-stretch as build

# Dependencies.
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install \
    ca-certificates \
    libssl-dev \
    pkg-config \
    libpq-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Create empty shell project.
RUN USER=root cargo new --bin ark_auth
WORKDIR /ark_auth

# Copy manifest, build dependencies.
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

# Copy source tree.
COPY ./migrations ./migrations
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/ark_auth*
RUN cargo build --release

# Final stage.
FROM rust:1.34.2-slim-stretch

# Dependencies.
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get -y install \
    ca-certificates \
    libpq-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy build artifact.
COPY --from=build /ark_auth/target/release/ark_auth .

# Set the startup command to run binary.
CMD ["./ark_auth", "start-server"]
