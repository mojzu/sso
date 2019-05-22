```rust,skt-list-ok
use manual::*;
use ark_auth::server::route::key;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-list-bad-request
use manual::*;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-list-forbidden
use manual::*;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```
