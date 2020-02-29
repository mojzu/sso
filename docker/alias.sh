# ```bash
# source docker/alias.sh
# ```
export UID
# Alias to run build container.
alias sso-build='docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" sso/build:v1'
# Alias to run build container with hostname.
sso-build-host() {
    local host="$1"
    shift 1
    docker run --rm -it --init --user $(id -u):$(id -g) --network compose -v "$(pwd):/build" --hostname $host --name $host sso/build:v1 "$@"
}
# Alias to run docker-compose build project.
alias sso-build-build='docker-compose -f docker/build.yml build'
# Alias to run docker-compose sso project.
alias sso='docker-compose -f docker/sso.yml -p sso'
# Alias to run mkdocs container.
alias sso-mkdocs='docker run --rm -it --user $(id -u):$(id -g) -p 8000:8000 -v "$(pwd):/build" sso/mkdocs:v1'
