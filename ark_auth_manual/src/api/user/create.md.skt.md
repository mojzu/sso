```rust,skt-create-ok
use ark_auth_manual::*;
use ark_auth::server::route::user;
use chrono::Utc;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-create-forbidden
use ark_auth_manual::*;
use ark_auth::server::route::user;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```
