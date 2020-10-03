FROM sso-build/sso:latest

COPY .devcontainer/test-ci/sso2.toml /config/sso.toml
