## OAuth2 Login

`$server_url/v1/auth/provider/github/oauth2`

Create service with OAuth2 provider URL and key, and start server.

```shell
mz_auth create-service-with-key $service_name $service_url \
    [--github-oauth2-url $service_github_oauth2_url] \
    [--microsoft-oauth2-url $service_microsoft_oauth2_url]
mz_auth start-server
```

Service creates a user with email address matching OAuth2 provider.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email"}' \
  $server_url/v1/user
```

Service creates a key for user.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

User makes OAuth2 login request to service.

Service requests a redirect URL for OAuth2 provider, supported providers are `github`, `microsoft`.

```shell
curl --header "Authorization: $service_key" \
  --request POST \
  $server_url/v1/auth/provider/$oauth2_provider/oauth2
```

Service redirects user to returned URL, OAuth2 provider authentication occurs.

If successful, OAuth2 provider redirects user to `$server_url/v1/auth/provider/$oauth2_provider/oauth2` with required query parameters.

Query parameters are exchanged for API access token, authenticated email address is requested from OAuth2 provider APIs.

If authenticated email returned by API matches a user email address, and user has key for specified service, a user token is produced and the user is redirected to `$service_url?access_token=$token&refresh_token=$token`.

Service receives access token and refresh token via query parameters. Service can verify access token to authenticate requests.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$access_token"}' \
  $server_url/v1/auth/token/verify
```

Refresh token can be used to refresh token.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$refresh_token"}' \
  $server_url/v1/auth/token/refresh
```

Access or refresh token can be revoked, this will disable the key created earlier and prevent verify and refresh.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  $server_url/v1/auth/token/revoke
```
