# ark_kv

Encrypted key, value storage library and binary crate.

-   **Asymmetric encryption**, key value data is encrypted, reads require a secret key.
-   **Compression**, key value data is compressed to reduce disk use.
-   **Integrity**, key value data is hashed on write, hash is checked on read.
-   **Versioning**, key value writes are versioned.
-   **Retention**, key value version retention is configurable (number of versions and/or duration of time).

## CLI

### create secret-key

```shell
ark_kv create secret-key SECRET_KEY
```

Create secret key and write to file `./secret.key`.

```Shell
ark_kv create secret-key ./secret.key
```

### verify secret-key

```shell
ark_kv verify secret-key SECRET_KEY
```

Verify ark key read from file `./secret.key`.

```shell
ark_kv verify secret-key ./secret.key
```

### create disk

```shell
ark_kv create disk DISK SECRET_KEY
```

Create a new disk named `Foo` with secret key read from file `./secret.key`.

```shell
ark_kv create disk Foo ./secret.key
```

Create a new disk named `Bar` with secret key read from file `./secret.key`, retain at least 10 key versions, retain all key versions for a duration of at least 1 week (604800 seconds).

```shell
ark_kv create disk Bar ./secret.key  --version-retention 10 --duration-retention 604800
```

### list

```Shell
ark_kv list [DISK] [KEY]
```

List all disks.

```Shell
ark_kv list
```
