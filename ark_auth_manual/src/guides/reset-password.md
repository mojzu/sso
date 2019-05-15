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
  $server_url/v1/auth/reset/password
```

Email containing URL is send to user email address, URL in format `$service_url?email=$user_email&reset_password_token=$token`.

Server receives token via query parameter and makes reset password confirm request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$token","password":"$user_password"}' \
  $server_url/v1/auth/reset/password/confirm
```

User makes login request to service, service makes a login request.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"email":"$user_email","password":"$user_password"}' \
  $server_url/v1/auth/login
```

## Test

```rust,skt-reset-password
let (service, service_key) = service_key_create(&client);
let user_email = user_email_create();

let url = server_url("/v1/user");
let request = user::CreateBody {
    name: "User Name".to_owned(),
    email: user_email.clone(),
    password: Some("guest".to_owned()),
};
let mut response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let body = response.json::<user::CreateResponse>().unwrap();
let user = body.data;
let status = response.status();
let content_type = header_get(&response, "content-type");
assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert_eq!(user.name, "User Name");
assert_eq!(user.email, user_email);

let url = server_url("/v1/key");
let request = key::CreateBody {
    name: "Key Name".to_owned(),
    service_id: None,
    user_id: Some(user.id),
};
let mut response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let body = response.json::<key::CreateResponse>().unwrap();
let user_key = body.data;
let status = response.status();
let content_type = header_get(&response, "content-type");
assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert_eq!(user_key.name, "Key Name");
assert_eq!(user_key.service_id.unwrap(), service.id);
assert_eq!(user_key.user_id.unwrap(), user.id);

let url = server_url("/v1/auth/reset/password");
let request = auth::reset::PasswordBody {
    email: user_email.clone(),
    template: None,
};
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 200);
assert_eq!(content_length, "0");
```

TODO(test): Test reset password confirm.
