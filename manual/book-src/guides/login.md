# Login

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
  --data '{"name":"$user_name","email":"$user_email","password":"$user_password","active":true}' \
  $server_url/v1/user
```

Service creates a key for user.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$key_name","user_id":$user_id}' \
  $server_url/v1/key
```

User makes login request to service, services makes a login request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

Service receives token response, token can be verified to authenticate requests.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$user_token"}' \
  $server_url/v1/auth/token/verify
```
