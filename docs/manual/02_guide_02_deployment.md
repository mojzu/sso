## Deployment

TODO(docs): How to deploy sso server.

### Nginx

Example [nginx][nginx] configuration.

```nginx
http {
    # HTTPS and headers configuration.
    # https://www.nginx.com/blog/using-free-ssltls-certificates-from-lets-encrypt-with-nginx/
    # https://www.owasp.org/index.php/OWASP_Secure_Headers_Project

    # API request rate limit.
    # https://www.nginx.com/blog/rate-limiting-nginx/
    limit_req_zone $binary_remote_addr zone=api_zone:10m rate=50r/s;

    # API proxy server.
    # https://docs.nginx.com/nginx/admin-guide/web-server/reverse-proxy/
    location /api {
        limit_req zone=api_zone burst=100 nodelay;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_pass http://127.0.0.1:4000;
    }
}
```

### Prometheus

Example [Prometheus][prometheus] configuration and alerting rules can be found in `docker/prometheus`.

```yml
scrape_configs:
    # ...

    - job_name: "sso"
      metrics_path: "/v1/metrics"
      bearer_token: "$root_or_service_key"
      static_configs:
          - targets: ["$server_url"]
```

```shell
curl --header "Authorization: $root_or_service_key" $server_url/v1/metrics
```
