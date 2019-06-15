# Ping [GET /v1/ping]

Uptime test route, authentication is not required for this route.

## Response [200, OK]

```json
"pong"
```

### Test

```rust
let client = create_client();
ping_server(&client);
```

```rust,skeptic-template
use manual::*;
fn main() {{
    {}
}}
```
