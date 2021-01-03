#!/bin/bash
# Install pacakges
apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends wget build-essential libssl-dev pkg-config 2>&1

# <https://github.com/rust-lang/docker-rust>
# <https://github.com/rust-lang/docker-rust/blob/master/1.49.0/buster/Dockerfile>
set -eux; \
dpkgArch="$(dpkg --print-architecture)"; \
case "${dpkgArch##*-}" in \
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='ed7773edaf1d289656bdec2aacad12413b38ad0193fff54b2231f5140a4b07c5' ;; \
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='7a7b9d246ad63358705d8d4a7d5c2ef1adfec24525d1d5c44a7739e1b867e84d' ;; \
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='f80a0a792b3ab905ab4919474daf4d3f60e574fc6987e69bfba2fd877241a8de' ;; \
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='4473c18286aa1831683a772706d9a5c98b87a61cc014d38063e00a63a480afef' ;; \
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
esac; \
url="https://static.rust-lang.org/rustup/archive/1.23.1/${rustArch}/rustup-init"; \
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
