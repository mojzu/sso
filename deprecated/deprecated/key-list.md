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
assert_eq!(data[0], service_key.id);
```

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
