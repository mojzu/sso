#!/bin/bash
set -e

# Print versions.
cat /etc/os-release
rustup --version
cargo --version
rustc --version
cargo make --version
diesel --version
cargo audit --version
go version
protoc --version
pandoc --version
