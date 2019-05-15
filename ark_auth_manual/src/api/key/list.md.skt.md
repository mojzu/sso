```rust,skt-list-ok
use ark_auth_manual::*;
use ark_auth::server::route::key;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-list-bad-request
use ark_auth_manual::*;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-list-forbidden
use ark_auth_manual::*;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```
