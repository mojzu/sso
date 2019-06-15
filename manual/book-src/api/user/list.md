# List [GET /v1/user]

List users.

## Request

```
?gt=X&lt=Y&limit=Z
```

- `gt`: Greater than ID (optional).
- `lt`: Less than ID (optional).
- `limit`: Limit number of returned items (optional).

## Response [200, OK]

```json
{
    "meta": {
        "gt": 0,
        "lt": null,
        "limit": 10
    },
    "data": [
        1,
        ...
    ]
}
```

### Meta

- `gt`: Greater than ID, or null.
- `lt`: Greater than ID, or null.
- `limit`: Returned items limit.

### Data

Array of IDs.

## Response [400, Bad Request]

- Request query is invalid.

## Response [403, Forbidden]

- Authorisation header is invalid.

### Test

```rust,skt-list-forbidden
let url = server_url("/v1/user");

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
