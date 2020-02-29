# OAuth2 Login

`$server_url/v1/auth/provider/github/oauth2`

Create service with OAuth2 provider URL and key, and start server.

```bash
sso-cli create-service-with-key $service_name $service_url \
    [--github-oauth2-url $service_github_oauth2_url] \
    [--microsoft-oauth2-url $service_microsoft_oauth2_url]
```

Service creates a user with email address matching OAuth2 provider.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email","locale":"en","timezone":"Etc/UTC"}' \
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

User makes OAuth2 login request to service, service requests a redirect URL for OAuth2 provider, supported providers are `github`, `microsoft`.

```bash
curl --header "Authorization: $service_key" \
  localhost:8042/v1/auth/provider/$oauth2_provider/oauth2
```

Service redirects user to returned URL, OAuth2 provider authentication occurs.

If successful, OAuth2 provider redirects user to `$service_$provider_oauth2_url?code=$code&state=$state`. Service receives query parameters for callback.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"code":"$code","state":"$state"}' \
  localhost:8042/v1/auth/provider/$oauth2_provider/oauth2
```

Query parameters are exchanged for API access token, authenticated email address is requested from OAuth2 provider APIs.

If authenticated email returned by API matches a user email address, and user has key for specified service, a user token is produced and the user is redirected to `$service_url?access_token=$token&refresh_token=$token`.

Service receives access token and refresh token via query parameters. Service can verify access token to authenticate requests.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$access_token"}' \
  localhost:8042/v1/auth/token/verify
```

Refresh token can be used to refresh token.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$refresh_token"}' \
  localhost:8042/v1/auth/token/refresh
```

Access or refresh token can be revoked.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token"}' \
  localhost:8042/v1/auth/token/revoke
```

This will disable and revoke the user key created earlier.
