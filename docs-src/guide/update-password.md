# Update Password

Create service with key and start server.

```bash
sso-cli create-service-with-key $service_name $service_url \
    --local-url $service_local_url
```

Service creates a user with password.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email","locale":"en","timezone":"Etc/UTC","password_allow_reset":true,"password_require_update":false,"password":"$user_password"}' \
  localhost:8042/v1/user
```

Service creates a key for user.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"type":"TOKEN","name":"$key_name","user_id":"$user_id"}' \
  localhost:8042/v1/key
```

User makes update password request to service, service makes an update password request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password","new_password":"$new_user_password"}' \
  localhost:8042/v1/auth/provider/local/update-password
```

User makes login request to service, service makes a login request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$new_user_password"}' \
  localhost:8042/v1/auth/provider/local/login
```

Email containing password update revoke URL is sent to user email address, URL in format `$service_provider_local_url?type=update_password&email=$user_email&token=$token`.

Service receives token via query parameter and makes update password revoke request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  localhost:8042/v1/auth/provider/local/update-password/revoke
```

This will disable the user, and disable and revoke all user keys.
