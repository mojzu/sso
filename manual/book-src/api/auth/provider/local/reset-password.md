# Reset Password [POST /v1/auth/provider/local/reset/password]

Reset password request for email address.

## Request

```json
{
  "email": "user@example.com"
}
```

- `email`: User email address (required).

## Response [200, OK]

### Test

```rust,skt-password-ok
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

// Unknown email address.
let url = server_url("/v1/auth/provider/local/reset/password");
let request = auth::reset::PasswordBody {
    email: user_email_create(),
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
assert_eq!(status, 400);
assert_eq!(content_length, "0");

// Service 2 cannot reset password.
let (_service2, service2_key) = service_key_create(&client);
let request = auth::reset::PasswordBody {
    email: user_email.clone(),
    template: None,
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

// Reset password success.
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

## Response [400, Bad Request]

- Request body is invalid.
- User email address is invalid or unknown.
- User is not assigned key for service.
- User password is null.
- Unable to send password reset email.

### Test

```rust,skt-password-bad-request
let (_service, service_key) = service_key_create(&client);
let url = server_url("/v1/auth/provider/local/reset/password");

// Invalid body (missing key property).
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

// Invalid body (invalid key property).
let request = json_value(r#"{ "email": "reset-password" }"#);
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
```

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-password-forbidden
let url = server_url("/v1/auth/provider/local/reset/password");
let request = auth::reset::PasswordBody {
    email: user_email_create(),
    template: None,
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

# Reset Password Confirm [POST /v1/auth/provider/local/reset/password/confirm]

Confirm reset password request.

## Request

```json
{
  "token": "eyJ0e...6eEvY",
  "password": "guest"
}
```

- `token`: JWT reset authentication token for user (required).
- `password`: User password (required).

## Response [200, OK]

```json
{
  "meta": {
    "password_strength": 2,
    "password_pwned": false
  }
}
```

### Meta

- `password_strength`: Password strength score by `zxcvbn`, null if scoring failed.
- `password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed or feature disabled.

## Response [400, Bad Request]

- Request body is invalid.
- Token is invalid or has been used.
- User password is invalid.
- User is not assigned key for service.

### Test

```rust,skt-password-confirm-bad-request
let (_service, service_key) = service_key_create(&client);
let url = server_url("/v1/auth/provider/local/reset/password/confirm");

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

// Invalid body (invalid token property).
let request = json_value(r#"{ "token": "", "password": "guest" }"#);
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
let request = json_value(r#"{ "token": "some-token", "password": "" }"#);
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
```

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-password-confirm-forbidden
let url = server_url("/v1/auth/provider/local/reset/password/confirm");
let request = auth::reset::PasswordConfirmBody {
    token: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9".to_owned(),
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
