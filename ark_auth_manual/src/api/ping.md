# Ping [GET /v1/ping]

Uptime test route, authentication is not required for this route.

## Response [200, OK]

```json
"pong"
```

```rust,skt-ping
let mut response = reqwest::get("http://localhost:9000/v1/ping").unwrap();
let body = response.json::<Value>().unwrap();
let status = response.status();
let content_length = header_get(&response, "content-length");
let content_type = header_get(&response, "content-type");

assert_eq!(status, 200);
assert_eq!(content_length, "6");
assert_eq!(content_type, "application/json");
assert_eq!(body, Value::String("pong".to_owned()))
```
