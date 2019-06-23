# Login [POST /v1/auth/provider/local/login]

Login with email address and password.

## Request

```json
{
  "email": "user@example.com",
  "password": "guest"
}
```

- `email`: User email address (required).
- `password`: User password (required).

## Response [200, OK]

```json
{
  "meta": {
    "password_strength": 2,
    "password_pwned": false
  },
  "data": {
    "user_id": 1,
    "token": "eyJ0e...6eEvY",
    "token_expires": 1555881550
  }
}
```

### Meta

- `password_strength`: Password strength score by `zxcvbn`, null if scoring failed.
- `password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed or feature disabled.

### Data

- `user_id`: User ID.
- `token`: JWT authentication token for user.
- `token_expires`: JWT expiry time, unix timestamp.

### Test

```rust,skt-login-ok
let (service, service_key) = service_key_create(&client);
let url = server_url("/v1/auth/provider/local/login");

// User not created, unknown email address.
let user_email = user_email_create();
let user_password = "guest".to_owned();
let request = auth::LoginBody {
    email: user_email.clone(),
    password: user_password.clone(),
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
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// User created, does not have key for service.
let user_url = server_url("/v1/user");
let request = user::CreateBody {
    name: "User Name".to_owned(),
    email: user_email.clone(),
    password: Some(user_password.clone()),
};
let mut response = client
    .post(&user_url)
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

let request = auth::LoginBody {
    email: user_email.clone(),
    password: user_password.clone(),
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
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Key created, incorrect password.
let key_url = server_url("/v1/key");
let request = key::CreateBody {
    name: "Key Name".to_owned(),
    service_id: None,
    user_id: Some(user.id),
};
let mut response = client
    .post(&key_url)
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

let request = auth::LoginBody {
    email: user_email.clone(),
    password: "guests".to_owned(),
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
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Service 2 does not have key for user.
let (_service2, service2_key) = service_key_create(&client);
let request = auth::LoginBody {
    email: user_email.clone(),
    password: user_password.clone(),
};
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service2_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Correct password.
let request = auth::LoginBody {
    email: user_email.clone(),
    password: user_password.clone(),
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
assert!(user_token.token.len() > 0);
assert!(user_token.token_expires > 0);
```

## Response [400, Bad Request]

- Request body is invalid.
- User email address is invalid or unknown.
- User password is invalid or incorrect or null.
- User is not assigned key for service.

### Test

```rust,skt-login-bad-request
let (service, service_key) = service_key_create(&client);
let url = server_url("/v1/auth/provider/local/login");

// Invalid body (missing properties).
let request = json_value(r#"{}"#);
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Invalid body (missing email property).
let request = json_value(r#"{ "email": "login@example.com" }"#);
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Invalid body (missing password property).
let request = json_value(r#"{ "password": "guest" }"#);
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Invalid body (invalid email property).
let request = json_value(r#"{ "email": "login", "password": "guest" }"#);
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Invalid body (invalid password property).
let request = json_value(r#"{ "email": "login@example.com", "password": "" }"#);
let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Disabled user.
let user_email = user_email_create();
let user = user_post_200(&service_key, "User Name", &user_email, false, Some("guest"));
key_post_user_200(&service, &service_key, &user, "Key Name");
auth_login_post_400(&service_key, &user_email, "guest");
```

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-login-forbidden
let url = server_url("/v1/auth/provider/local/login");
let request = auth::LoginBody {
    email: user_email_create(),
    password: "guest".to_owned(),
};

let response = client
    .post(&url)
    .header("content-type", "application/json")
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 403);
assert_eq!(content_length, "0");

let response = client
    .post(&url)
    .header("content-type", "application/json")
    .header("authorization", "some-invalid-key")
    .json(&request)
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 403);
assert_eq!(content_length, "0");
```
