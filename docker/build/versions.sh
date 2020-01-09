#!/bin/bash
set -e

# Print versions.
printf "os\n"
cat /etc/os-release
printf "\nrustup\n"
rustup --version
printf "\ncargo\n"
cargo --version
printf "\nrustc\n"
rustc --version
printf "\ncargo-make\n"
cargo make --version
printf "\ndiesel\n"
diesel --version
printf "\ncargo-audit\n"
cargo audit --version
printf "\ngo\n"
go version
printf "\nprotoc\n"
protoc --version
printf "\npandoc\n"
pandoc --version
