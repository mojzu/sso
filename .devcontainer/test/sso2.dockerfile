FROM sso-build/sso:latest

COPY .devcontainer/test/sso2.toml /config/sso.toml
COPY .devcontainer/test/sso2_template.hbs /config/template_html.hbs
