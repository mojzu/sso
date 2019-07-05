## Test

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
