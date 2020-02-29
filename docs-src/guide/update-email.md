# Update Email

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

User makes update email request to service, service makes an update email request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password","new_email":"$new_user_email"}' \
  localhost:8042/v1/auth/provider/local/update-email
```

User makes login request to service, service makes a login request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$new_user_email","password":"$user_password"}' \
  localhost:8042/v1/auth/provider/local/login
```

Email containing email update revoke URL is sent to old user email address, URL in format `$service_provider_local_url?type=update_email&email=$user_email&old_email=$user_old_email&token=$token`.

Service receives token via query parameter and makes update email revoke request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  localhost:8042/v1/auth/provider/local/update-email/revoke
```

This will disable the user, and disable and revoke all user keys.
