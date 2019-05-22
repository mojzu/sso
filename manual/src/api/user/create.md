# Create [POST /v1/user]

Create user.

## Request

```json
{
  "name": "User Name",
  "email": "user@example.com",
  "password": "guest"
}
```

- `name`: User name (required).
- `email`: User email address, must be unique (required).
- `password`: User password, optional.

## Response [200, OK]

```json
{
  "meta": {
    "password_strength": 0,
    "password_pwned": true
  },
  "data": {
    "created_at": "2019...04Z",
    "updated_at": "2019...04Z",
    "id": 10,
    "name": "User Name",
    "email": "user@example.com"
  }
}
```

### Meta

- `password_strength`: Password strength score by `zxcvbn`, null if scoring failed or password was not provided.
- `password_pwned`: True if password detected by `haveibeenpwned.com`, null if request failed, feature disabled or password was not provided.

### Data

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: User ID.
- `name`: User name.
- `email`: User email address.

### Test

```rust,skt-create-ok
let (_service, service_key) = service_key_create(&client);
let url = server_url("/v1/user");
let user_email = user_email_create();
let before = Utc::now();

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
assert!(user.created_at.gt(&before));
assert!(user.updated_at.gt(&before));
assert!(user.id > 0);
assert_eq!(user.name, "User Name");
assert_eq!(user.email, user_email);
assert!(user.password_hash.is_none());
assert!(user.password_revision.is_none());
```

## Response [400, Bad Request]

- Request body is invalid.
- User name is invalid.
- User email address is invalid or not unique.
- User password is invalid.

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-create-forbidden
let url = server_url("/v1/user");
let request = user::CreateBody {
    name: "User Name".to_owned(),
    email: user_email_create(),
    password: None,
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
