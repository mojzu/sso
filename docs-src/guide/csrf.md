# CSRF

Create service with key and start server.

```bash
sso create-service-with-key $service_name $service_url
sso start-server
```

Service creates a CSRF token.

```bash
curl --header "Authorization: $service_key" \
  $server_url/v1/auth/csrf
```

Service verifies CSRF token.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$csrf_key"}' \
  $server_url/v1/auth/csrf
```
