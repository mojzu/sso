# OAuth Login

Initialise a service and start server.

```Shell
$ ark_auth init $service_name $service_url
$ ark_auth start
```

Service creates a user with email address matching OAuth provider.

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

User makes OAuth login request to service.

Service requests a redirect URL for OAuth provider, supported providers are `github`, `microsoft`.

```Shell
$ curl --header "Authorization: $service_key" \
  --request POST \
  $server_url/v1/auth/oauth/$oauth_provider
```

Service redirects user to returned URL, OAuth provider authentication occurs.

If successful, OAuth provider redirects user to `$server_url/v1/auth/oauth/$oauth_provider` with required query parameters.

Query parameters are exchanged for API access token, authenticated email address is requested from OAuth provider APIs.

If authenticated email returned by API matches a user email address, and user has key for specified service, a user authentication token is generated and the user is redirected to `$service_url?token=$token`.

Service receives token via query parameter and verifies it to authenticate requests.

```Shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  $server_url/v1/auth/token/verify
```
