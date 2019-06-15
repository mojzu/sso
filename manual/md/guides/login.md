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
  --data '{"name":"$user_name","email":"$user_email","password":"$user_password"}' \
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

## Test

```rust
let mut client = create_client();
let (service, service_key) = create_service_key(&client);
let user_email = create_user_email();

client.options.set_authorisation(&service_key.value);
let user = create_user(&client, "User Name", &user_email, true, Some("guest"));
let _user_key = create_user_key(&client, "Key Name", service.id, user.id);

let user_token = local_login(&client, user.id, &user_email, "guest");
verify_user_token(&client, &user_token);
```

```rust,skeptic-template
use manual::*;
fn main() {{
    {}
}}
```
