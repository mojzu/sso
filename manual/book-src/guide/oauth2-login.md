# OAuth2 Login

Create service with key and start server.

```shell
$ ark_auth create-service-with-key $service_name $service_url
$ ark_auth start-server
```

Service creates a user with email address matching OAuth2 provider.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_active":true,"name":"$user_name","email":"$user_email"}' \
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

User makes OAuth2 login request to service.

Service requests a redirect URL for OAuth2 provider, supported providers are `github`, `microsoft`.

```shell
$ curl --header "Authorization: $service_key" \
  --request POST \
  $server_url/v1/auth/provider/$oauth2_provider/oauth2
```

Service redirects user to returned URL, OAuth2 provider authentication occurs.

If successful, OAuth2 provider redirects user to `$server_url/v1/auth/provider/$oauth2_provider/oauth2` with required query parameters.

Query parameters are exchanged for API access token, authenticated email address is requested from OAuth2 provider APIs.

If authenticated email returned by API matches a user email address, and user has key for specified service, a user authentication token is generated and the user is redirected to `$service_url?token=$token`.

Service receives token via query parameter and verifies it to authenticate requests.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$user_token"}' \
  $server_url/v1/auth/token/verify
```
