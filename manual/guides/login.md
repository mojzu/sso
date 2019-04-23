# Login

Initialise a service and start server.

```Shell
$ ark_auth init $service_name $service_url
$ ark_auth start
```

Service creates a user with password.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$user_name","email":"$user_email","password":"$user_password"}' \
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

User makes login request to service, service makes a login request.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/login
```

Service receives token response and verifies it to authenticate requests.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$user_token"}' \
  $server_url/v1/auth/token/verify
```
