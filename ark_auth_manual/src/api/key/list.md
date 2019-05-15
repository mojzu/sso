# List [GET /v1/key]

List keys.

## Request

Query parameters: `?gt=X&lt=Y&limit=Z`

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
        {
            "created_at": "2019...90Z",
            "updated_at": "2019...90Z",
            "id": 1,
            "name": "Key Name",
            "value": "667...35c",
            "service_id": 1,
            "user_id": null
        },
        ...
    ]
}
```

### Meta

- `gt`: Greater than ID, or null.
- `lt`: Less than ID, or null.
- `limit`: Returned items limit.

### Data

Array of read items.

- `created_at`: Created time ISO 8601 timestamp.
- `updated_at`: Updated time ISO 8601 timestamp.
- `id`: Key ID.
- `name`: Key name.
- `value`: Key value.
- `service_id`: Key service ID relation or null.
- `user_id`: Key user ID relation or null.

### Test

```rust,skt-list-ok
let (_service, service_key) = service_key_create(&client);
let url = server_url("/v1/key");

let mut response = client
    .get(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .send()
    .unwrap();
let body = response.json::<key::ListResponse>().unwrap();
let meta = body.meta;
let data = body.data;
let status = response.status();
let content_type = header_get(&response, "content-type");
assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert_eq!(meta.gt, Some(0));
assert_eq!(meta.lt, None);
assert_eq!(meta.limit, 10);
assert_eq!(data.len(), 1);

let body_key = &data[0];
assert!(body_key.created_at.eq(&service_key.created_at));
assert!(body_key.updated_at.eq(&service_key.updated_at));
assert_eq!(body_key.id, service_key.id);
assert_eq!(body_key.name, service_key.name);
assert_eq!(body_key.value, service_key.value);
assert_eq!(body_key.service_id, service_key.service_id);
assert_eq!(body_key.user_id, service_key.user_id);
```

## Response [400, Bad Request]

- Request query is invalid.

### Test

```rust,skt-list-bad-request
let (_service, service_key) = service_key_create(&client);
let url = server_url("/v1/key");

let response = client
    .get(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .query(&[("gt", "-1")])
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

let response = client
    .get(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .query(&[("lt", "-1")])
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");

let response = client
    .get(&url)
    .header("content-type", "application/json")
    .header("authorization", service_key.value.clone())
    .query(&[("limit", "-1")])
    .send()
    .unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
assert_eq!(status, 400);
assert_eq!(content_length, "0");
```

## Response [403, Forbidden]

- Authorisation header is missing or invalid.

### Test

```rust,skt-list-forbidden
let url = server_url("/v1/key");

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
