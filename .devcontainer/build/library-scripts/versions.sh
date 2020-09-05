#!/bin/bash
set -e

# Print versions.
printf "os\n"
cat /etc/os-release

printf "\ndocker\n"
docker --version
printf "\ndocker-compose\n"
docker-compose --version

printf "\nrustup\n"
rustup --version
printf "\ncargo\n"
cargo --version
printf "\nrustc\n"
rustc --version
printf "\ncargo-make\n"
cargo make --version
printf "\ncargo-audit\n"
cargo audit --version

printf "\nnode\n"
node --version
printf "\nyarn\n"
yarn --version
printf "\nnpm-check-updates\n"
npm-check-updates --version
