# Deployment

## Kubernetes

Example [Kubernetes][kubernetes] configuration files can be found in the `kubernetes` directory of the repository.

## Prometheus

Example [Prometheus][prometheus] configuration and alerting rules can be found in the `docker/prometheus` directory of the repository.

## Traefik

Example [Traefik][traefik] configuration files can be found in the `docker/traefik` directory of the repository.

- Can apply [rate limit][traefik-ratelimit].
- Can use [forward authentication][traefik-forwardauth] with `SSO_TRAEFIK` environment variable.
- Can use [router TLS][traefik-routers-tls] or `SSO_TLS_*` environment variables for encrypting traffic.

[kubernetes]: https://kubernetes.io/
[prometheus]: https://prometheus.io/
[traefik]: https://docs.traefik.io
[traefik-ratelimit]: https://docs.traefik.io/middlewares/ratelimit/
[traefik-forwardauth]: https://docs.traefik.io/middlewares/forwardauth/
[traefik-routers-tls]: https://docs.traefik.io/routing/routers/#tls

## CORS

CORS for `sso-openapi` binary can be configured using `SSO_CORS_ALLOW_ORIGIN` environment variable.

- If variable is undefined or empty string, defaults to allow all CORS requests (allow origin `*` and handle preflight requests).
- If variable is not empty, is treated as a comma separated list of acceptable origin values. If origin is undefined or empty request is not handled. If origin matches then allow origin header is et and preflight requests are handled.
