## Register

Create service with key and start server.

```bash
sso create-service-with-key $service_name $service_url \
    --allow-register true \
    --email-text "This is appended to outgoing emails." \
    --local-url $service_local_url
sso start-server
```

User makes a register request to service, service makes register request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$user_name","email:"$user_email","locale":"en","timezone":"Etc/UTC"}' \
  $server_url/v1/auth/provider/local/register
```

Email containing URL is sent to user email address, URL in format: `$service_provider_local_url?type=register&email=$user_email&token=$token`.

Service receives token via query parameter and makes register confirm request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token","password":"$user_password","password_allow_reset":false}' \
  $server_url/v1/auth/provider/local/register/confirm
```

User makes login request to service, service makes a login request.

```bash
curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

TODO(sam,docs): Revoke email documentation.
