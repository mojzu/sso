## Update Password

Create service with key and start server.

```shell
ark_auth create-service-with-key $service_name $service_url
ark_auth start-server
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

User makes update password request to service, service makes an update password request.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key","password":"$user_password","new_password":"$new_user_password"}' \
  $server_url/v1/auth/provider/local/update/password
```

User makes login request to service, service makes a login request.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$new_user_password"}' \
  $server_url/v1/auth/provider/local/login
```

After successful update, an email containing URL is sent to user email address, URL in format: `$service_url?email=$email&update_password_token=$token`.

If user opens URL, service receives token via query parameter and makes update password revoke request, this will disable user and all linked keys and prevent login.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  $server_url/v1/auth/provider/local/update/password/revoke
```
