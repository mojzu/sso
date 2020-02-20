## API Key

Create service with key.

```bash
sso-cli create-service-with-key $service_name $service_url
```

Service creates a user without password.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email"}' \
  sso.localhost/api/v1/user
```

Service creates a key for user.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"type":"KEY","name":"$key_name","user_id":"$user_id"}' \
  sso.localhost/api/v1/key
```

User makes requests to service with key value, key can be verified to authenticate requests.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  sso.localhost/api/v1/auth/key/verify
```

Key can be revoked, this will disable and revoke the key created earlier and prevent verification.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  sso.localhost/api/v1/auth/key/revoke
```
