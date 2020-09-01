FROM sso/sso:latest

COPY ./sso_config/sso.toml /config/sso.toml
COPY ./sso_config/template_html.hbs /config/template_html.hbs
