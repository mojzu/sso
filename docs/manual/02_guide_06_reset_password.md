## Reset Password

Create service with key and start server.

```bash
sso create-service-with-key $service_name $service_url \
    --local-url $service_local_url
sso start-server
```

Service creates a user with password.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email","locale":"en","timezone":"Etc/UTC","password_allow_reset":true,"password_require_update":false,"password":"$user_password"}' \
  $server_url/v1/user
```

Service creates a key for user.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"type":"Token","name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

User makes reset password request to service, service makes a reset password request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email"}' \
  $server_url/v1/auth/provider/local/reset-password
```

Email containing URL is sent to user email address, URL in format `$service_provider_local_url?type=reset_password&email=$user_email&token=$token`.

Service receives token via query parameter and makes reset password confirm request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/reset-password/confirm
```

User makes login request to service, service makes a login request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

TODO(docs): Revoke email documentation.
