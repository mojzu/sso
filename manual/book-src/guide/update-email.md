# Update Email

Create service with key and start server.

```shell
$ ark_auth create-service-with-key $service_name $service_url
$ ark_auth start-server
```

Service creates a user with password.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_active":true,"name":"$user_name","email":"$user_email","password":"$user_password"}' \
  $server_url/v1/user
```

Service creates a key for user.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_active":true,"name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

User makes update email request to service, service makes an update email request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key","password":"$user_password","new_email":"$new_user_email"}' \
  $server_url/v1/auth/provider/local/update/email
```

User makes login request to service, service makes a login request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$new_user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

After successful update, an email containing URL is sent to the old user email address, URL in format: `$service_url?email=$email&update_email_token=$token`.

If user opens URL, service receives token via query parameter and makes update email revoke request, this will disable user and all linked keys and prevent login.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  $server_url/v1/auth/provider/local/update/email/revoke
```
