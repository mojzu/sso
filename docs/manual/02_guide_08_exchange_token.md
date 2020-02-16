## Exchange Token

Create services 1 and 2 with keys.

```bash
sso-cli create-service-with-key $service_name2 $service_url2 \
    --local-url $service_local_url2
sso-cli create-service-with-key $service_name2 $service_url2 \
    --local-url $service_local_url2
```

Service 1 creates a user with password.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key1" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email","locale":"en","timezone":"Etc/UTC","password_allow_reset":true,"password_require_update":false,"password":"$user_password"}' \
  sso.localhost/api/v1/user
```

Services 1 and 2 create keys for user.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key1" \
  --request POST \
  --data '{"is_enabled":true,"type":"TOKEN","name":"$key_name","user_id":"$user_id"}' \
  sso.localhost/api/v1/key
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key2" \
  --request POST \
  --data '{"is_enabled":true,"type":"TOKEN","name":"$key_name","user_id":"$user_id"}' \
  sso.localhost/api/v1/key
```

User makes login request to service 1, service 1 makes a login request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key1" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  sso.localhost/api/v1/auth/provider/local/login
```

Service 1 receives access and refresh token response, sends service 2 copy of refresh token via query parameter.

Service 2 receives refresh token and makes exchange token request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key2" \
  --request POST \
  --data '{"token":"$refresh_token"}' \
  sso.localhost/api/v1/auth/token/exchange
```

Service 2 receives access and refresh token response, access token can be verified to authenticate requests.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key2" \
  --request POST \
  --data '{"token":"$access_token"}' \
  sso.localhost/api/v1/auth/token/verify
```
