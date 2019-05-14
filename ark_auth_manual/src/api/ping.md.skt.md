```rust,skt-ping
use serde_json::Value;

fn header_get<'a>(response: &'a reqwest::Response, name: &str) -> &'a str {{
    response.headers().get(name).unwrap().to_str().unwrap()
}}

fn main() {{
    {}
}}
```
