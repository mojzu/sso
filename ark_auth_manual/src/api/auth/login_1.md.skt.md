```rust,skt-login-ok
use ark_auth_manual::*;
use ark_auth::server::route::auth;
use ark_auth::server::route::user;
use ark_auth::server::route::key;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-login-bad-request
use ark_auth_manual::*;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-login-forbidden
use ark_auth_manual::*;
use ark_auth::server::route::auth;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```
