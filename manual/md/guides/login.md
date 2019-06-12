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
let (service, service_key) = service_key_create(&client);
let user_email = user_email_create();

let url = server_url("/v1/user");
let request = user::CreateBody {
    name: "User Name".to_owned(),
    email: user_email.clone(),
    active: true,
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
assert!(user.id > 0);
assert_eq!(user.name, "User Name");
assert_eq!(user.email, user_email);
assert!(user.password_hash.is_none());
assert!(user.password_revision.is_none());

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

let url = server_url("/v1/auth/login");
let request = auth::LoginBody {
    email: user_email.clone(),
    password: "guest".to_owned(),
};
let mut response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let body = response.json::<auth::LoginResponse>().unwrap();
let user_token = body.data;
let status = response.status();
let content_type = header_get(&response, "content-type");
assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert_eq!(user_token.user_id, user.id);

let url = server_url("/v1/auth/token/verify");
let request = auth::TokenBody {
    token: user_token.token.clone(),
};
let mut response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let body = response.json::<auth::TokenResponse>().unwrap();
let user_token_verify = body.data;
let status = response.status();
let content_type = header_get(&response, "content-type");
assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert_eq!(user_token_verify.user_id, user_token.user_id);
assert_eq!(user_token_verify.token, user_token.token);
assert_eq!(user_token_verify.token_expires, user_token.token_expires);
```
