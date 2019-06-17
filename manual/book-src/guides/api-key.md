# API Key

Create service with key and start server.

```shell
$ ark_auth create-service-with-key $service_name $service_url
$ ark_auth start-server
```

Service creates a user without password.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$user_name","email":"$user_email","active":true}' \
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

User makes requests to service with key value, key can be verified to authenticate requests.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"key":"$user_key"}' \
  $server_url/v1/auth/key/verify
```

## Test

```rust
let mut client = create_client();
let (service, service_key) = create_service_key(&client);
let user_email = create_user_email();

client.options.set_authorisation(&service_key.value);
let user = create_user(&client, "User Name", &user_email, true, None);
let user_key = create_user_key(&client, "Key Name", service.id, user.id);

verify_user_key(&client, &user_key);
```

```rust,skeptic-template
use manual::*;
fn main() {{
    {}
}}
```
