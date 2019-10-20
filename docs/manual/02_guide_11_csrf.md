## CSRF

Create service with key and start server.

```shell
sso create-service-with-key $service_name $service_url
sso start-server
```

Service creates a CSRF token.

```shell
curl --header "Authorization: $service_key" \
  $server_url/v1/auth/csrf
```

Service verifies CSRF token.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$csrf_key"}' \
  $server_url/v1/auth/csrf
```
