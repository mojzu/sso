## TOTP

Create service with key and start server.

```shell
sso create-service-with-key $service_name $service_url
sso start-server
```

Service creates a user (password optional).

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"name":"$user_name","email":"$user_email","locale":"$user_locale","timezone":"$user_timezone"}' \
  $server_url/v1/user
```

Service creates a TOTP key for user.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"is_enabled":true,"allow_key":false,"allow_token":false,"allow_totp":true,"name":"$key_name","user_id":"$user_id"}' \
  $server_url/v1/key
```

Generate QR code using the tool: <https://freeotp.github.io/qrcode.html>

- Issuer: sso
- Account: $user_email
- Secret: $user_key_value
- Digits: 6
- SHA1
- Timeout
- 30s

Import the QR code into the application: <https://freeotp.github.io/>

User makes request to service with TOTP code, service verifies TOTP code.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"user_id":"$user_id","totp":"$totp_code"}' \
  $server_url/v1/auth/totp
```

Key can be revoked, this will disable the key created earlier and prevent TOTP verification.

```shell
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  $server_url/v1/auth/key/revoke
```
