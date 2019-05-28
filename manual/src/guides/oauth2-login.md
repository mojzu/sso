# OAuth2 Login

Create service with key and start server.

```shell
$ ark_auth create-service-with-key $service_name $service_url
$ ark_auth start-server
```

Service creates a user with email address matching OAuth2 provider.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"name":"$user_name","email":"$user_email"}' \
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

User makes OAuth2 login request to service.

Service requests a redirect URL for OAuth2 provider, supported providers are `github`, `microsoft`.

```shell
$ curl --header "Authorization: $service_key" \
  --request POST \
  $server_url/v1/auth/oauth2/$oauth2_provider
```

Service redirects user to returned URL, OAuth2 provider authentication occurs.

If successful, OAuth2 provider redirects user to `$server_url/v1/auth/oauth2/$oauth2_provider` with required query parameters.

Query parameters are exchanged for API access token, authenticated email address is requested from OAuth2 provider APIs.

If authenticated email returned by API matches a user email address, and user has key for specified service, a user authentication token is generated and the user is redirected to `$service_url?token=$token`.

Service receives token via query parameter and verifies it to authenticate requests.

```shell
$ curl --header "Content-Type: application/json" \
  --header "Authorization: $service_key" \
  --request POST \
  --data '{"token":"$user_token"}' \
  $server_url/v1/auth/token/verify
```

## Test

```rust,skt-oauth2-login
let (service, service_key) = service_key_create(&client);
let user_email = user_email_create();

let url = server_url("/v1/user");
let request = user::CreateBody {
    name: "User Name".to_owned(),
    email: user_email.clone(),
    password: None,
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

let url = server_url("/v1/auth/oauth2/microsoft");
let mut response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .send()
    .unwrap();
let body = response.json::<auth::oauth2::UrlResponse>().unwrap();
let redirect_url = body.url;
let status = response.status();
let content_type = header_get(&response, "content-type");
assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert!(redirect_url.len() > 0);
```

TODO(test): Test OAuth2 flow, user email in initial request?
