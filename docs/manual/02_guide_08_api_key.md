## API Key

Create service with key and start server.

```shell
sso create-service-with-key $service_name $service_url \
    [--local-url $service_local_url]
sso start-server
```

Service creates a user without password.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email"}' \
  $server_url/v1/user
```

Service creates a key for user.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"type":"Key","name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

User makes requests to service with key value, key can be verified to authenticate requests.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  $server_url/v1/auth/key/verify
```

Key can be revoked, this will disable and revoke the key created earlier and prevent verification.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  $server_url/v1/auth/key/revoke
```
