#!/bin/bash
# Install pacakges
apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends wget build-essential libssl-dev pkg-config 2>&1

# <https://github.com/rust-lang/docker-rust>
# <https://github.com/rust-lang/docker-rust/blob/master/1.48.0/buster/Dockerfile>
set -eux; \
dpkgArch="$(dpkg --print-architecture)"; \
case "${dpkgArch##*-}" in \
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='ee7ade44063c96c6a37012cc599cb560dce95205c86d17b247c726d2285b230c' ;; \
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='afab10b89436bfb5ff7db4f4d5ad4d82faee98165915801d73e965e873661b1c' ;; \
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='99f42ab89c790e8825d91c99edee22b0176f68969d42dc27a89a5d9651c3ceba' ;; \
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='6fefdd5c429545b9c710c5492402215e1256cb002f19840db64303281b5ded3c' ;; \
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
esac; \
url="https://static.rust-lang.org/rustup/archive/1.23.0/${rustArch}/rustup-init"; \
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
cargo install --force cargo-audit --version "~0.13" \
&& cargo install --force cargo-make --version "~0.32" \
&& cargo install --force cargo-sort-ck --version "^2.1"
