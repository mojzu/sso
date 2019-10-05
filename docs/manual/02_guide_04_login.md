## Login

Create service with key and start server.

```shell
sso create-service-with-key $service_name $service_url
sso start-server
```

Service creates a user with password.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email","password":"$user_password"}' \
  $server_url/v1/user
```

Service creates a key for user.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

User makes login request to service, services makes a login request.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

Service receives token response, access token can be verified to authenticate requests.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$access_token"}' \
  $server_url/v1/auth/token/verify
```

Refresh token can be used to refresh token.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$refresh_token"}' \
  $server_url/v1/auth/token/refresh
```

Access or refresh token can be revoked, this will disable the key created earlier and prevent verify and refresh.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  $server_url/v1/auth/token/revoke
```
