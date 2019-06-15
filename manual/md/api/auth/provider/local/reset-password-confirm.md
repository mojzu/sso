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
