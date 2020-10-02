FROM sso/sso:latest

COPY .devcontainer/test/sso_config/sso.toml /config/sso.toml
COPY .devcontainer/test/sso_config/template_html.hbs /config/template_html.hbs
