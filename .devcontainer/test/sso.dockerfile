FROM sso-build/sso:latest

COPY .devcontainer/test/sso.toml /config/sso.toml
COPY .devcontainer/test/sso_template.hbs /config/template_html.hbs
