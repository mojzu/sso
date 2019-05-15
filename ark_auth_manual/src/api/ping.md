# Ping [GET /v1/ping]

Uptime test route, authentication is not required for this route.

## Response [200, OK]

```json
"pong"
```

### Test

```rust,skt-ping
let url = server_url("/v1/ping");
let mut response = client.get(&url).send().unwrap();

let body = response.json::<Value>().unwrap();
let status = response.status();
let content_type = header_get(&response, "content-type");

assert_eq!(status, 200);
assert_eq!(content_type, "application/json");
assert_eq!(body, Value::String("pong".to_owned()));
```
