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
  $server_url/v1/auth/login
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

```rust,skt-login
let (_service, service_key) = service_key_create(&client);

let url = server_url("/v1/user");
let mut response = client.post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value)
    .json(r#"{}"#)
    .send()
    .unwrap();
```

TODO(doc)

println!("response {:?}", response);
println!("body {:?}", body);
let body = response.json::<Value>().unwrap();
