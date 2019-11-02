## Update Email

Create service with key and start server.

```bash
sso create-service-with-key $service_name $service_url \
    [--local-url $service_local_url]
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

User makes update email request to service, service makes an update email request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"user_id":"$user_id","password":"$user_password","new_email":"$new_user_email"}' \
  $server_url/v1/auth/provider/local/update-email
```

User makes login request to service, service makes a login request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$new_user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

After successful update, an email containing URL is sent to the old user email address, URL in format: `$service_provider_local_url?type=update_email&email=$email&old_email=$old_email&token=$token`.

If user opens URL, service receives token via query parameter and makes update email revoke request, this will disable user and all linked keys and prevent login.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  $server_url/v1/auth/provider/local/update-email/revoke
```
