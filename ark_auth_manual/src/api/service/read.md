# Read [GET /v1/service/{id}]

Read service by ID.

## Response [200, OK]

```json
{
  "data": {
    "created_at": "2019...90Z",
    "updated_at": "2019...90Z",
    "id": 1,
    "name": "Service Name",
    "url": "https://..."
  }
}
```

### Data

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: Service ID.
- `name`: Service name.
- `url`: Service URL.

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-read-forbidden
let url = server_url("/v1/service/1");

let response = client
    .get(&url)
    .header("content-type", "application/json")
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 403);
assert_eq!(content_length, "0");

let response = client
    .get(&url)
    .header("content-type", "application/json")
    .header("authorization", "some-invalid-key")
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 403);
assert_eq!(content_length, "0");
```
