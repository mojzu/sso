FROM sso/sso:latest

COPY ./sso2_config/sso.toml /config/sso.toml
COPY ./sso2_config/template_html.hbs /config/template_html.hbs
