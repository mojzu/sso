# Reset Password

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

User makes reset password request to service, services make a reset password request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email"}' \
  $server_url/v1/auth/provider/local/reset/password
```

Email containing URL is send to user email address, URL in format `$service_url?email=$user_email&reset_password_token=$token`.

Server receives token via query parameter and makes reset password confirm request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/reset/password/confirm
```

User makes login request to service, service makes a login request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/provider/local/login
```

## Test

```rust
let mut client = create_client();
let (service, service_key) = create_service_key(&client);
let user_email = create_user_email();

client.options.set_authorisation(&service_key.value);
let user = create_user(&client, "User Name", &user_email, true, Some("guest"));
let _user_key = create_user_key(&client, "Key Name", service.id, user.id);

local_password_reset(&client, &user_email);
// TODO(test): Test reset password confirm, how to get reset password token.
```

```rust,skeptic-template
use manual::*;
fn main() {{
    {}
}}
```
