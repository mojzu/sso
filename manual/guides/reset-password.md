# Reset Password

Initialise a service and start server.

```Shell
$ ark_auth init $service_name $service_url
$ ark_auth start
```

Service creates a user without password.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$user_name","email":"$user_email"}' \
  $server_url/v1/user
```

Service creates a key for user.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$key_name","user_id":$user_id}' \
  $server_url/v1/key
```

User makes reset password request to service, services make a reset password request.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email"}' \
  $server_url/v1/auth/reset/password
```

Email containing URL is send to user email address, URL in format `$service_url?email=$user_email&reset_password_token=$token`.

Server receives token via query parameter and makes reset password confirm request.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token","password":"$user_password"}' \
  $server_url/v1/auth/reset/password/confirm
```

User makes login request to service, service makes a login request.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/login
```
