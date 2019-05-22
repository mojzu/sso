# Create [POST /v1/key]

Create key.

## Request

```json
{
    "name": "Name",
    "user_id": 2
}
```

- `name`: Key name (required).
- `user_id`: User ID (required).

## Response [200, OK]

```json
{
    "data": {
        "created_at": "2019...92Z",
        "updated_at": "2019...92Z",
        "id": 2,
        "name": "Key Name",
        "value": "84b...dc6",
        "service_id": 1,
        "user_id": 1
    }
}
```

### Data

Created key.

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: Key ID.
- `name`: Key name.
- `value`: Key value.
- `service_id`: Key service ID relation.
- `user_id`: Key user ID relation or null.

## Response [400, Bad Request]

- Request body is invalid.
- Key name is invalid.
- User ID is invalid or user does not exist.

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-create-forbidden
let url = server_url("/v1/key");
let request = key::CreateBody {
    name: "Key Name".to_owned(),
    service_id: None,
    user_id: Some(1)
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
