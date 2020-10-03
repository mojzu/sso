#!/bin/bash
# Install pacakges
apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends wget build-essential libssl-dev pkg-config 2>&1

# <https://github.com/rust-lang/docker-rust>
# <https://github.com/rust-lang/docker-rust/blob/master/1.46.0/buster/Dockerfile>
set -eux; \
dpkgArch="$(dpkg --print-architecture)"; \
case "${dpkgArch##*-}" in \
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='49c96f3f74be82f4752b8bffcf81961dea5e6e94ce1ccba94435f12e871c3bdb' ;; \
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='5a2be2919319e8778698fa9998002d1ec720efe7cb4f6ee4affb006b5e73f1be' ;; \
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='d93ef6f91dab8299f46eef26a56c2d97c66271cea60bf004f2f088a86a697078' ;; \
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e3d0ae3cfce5c6941f74fed61ca83e53d4cd2deb431b906cbd0687f246efede4' ;; \
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
esac; \
url="https://static.rust-lang.org/rustup/archive/1.22.1/${rustArch}/rustup-init"; \
wget "$url"; \
echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
chmod +x rustup-init; \
./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
rm rustup-init; \
chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
rustup --version; \
cargo --version; \
rustc --version;

# Configure apt and install packages
# <https://github.com/microsoft/vscode-dev-containers>
# <https://github.com/microsoft/vscode-dev-containers/blob/master/containers/rust/.devcontainer/base.Dockerfile>
apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends apt-utils dialog 2>&1 \
    && apt-get -y install git openssh-client less iproute2 procps lsb-release \
    && apt-get install -y lldb python3-minimal libpython3.7 \
    && rustup update 2>&1 \
    && rustup component add rls rust-analysis rust-src rustfmt clippy 2>&1

# depend: Install Cargo crates
# todo: Fix compilation error for cargo-sort-ck 2.1.1
cargo install --force cargo-audit --version "~0.12" \
&& cargo install --force cargo-make --version "~0.32" \
&& cargo install --force cargo-sort-ck --version "2.1.0"
