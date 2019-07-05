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
