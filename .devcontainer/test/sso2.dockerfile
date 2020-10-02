FROM sso/sso:latest

COPY .devcontainer/test/sso2_config/sso.toml /config/sso.toml
COPY .devcontainer/test/sso2_config/template_html.hbs /config/template_html.hbs
