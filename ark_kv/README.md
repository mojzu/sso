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

```shell
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
ark_kv create disk DISK SECRET_KEY [--version-retention X] [--duration-retention X]
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

```shell
ark_kv list [DISK] [KEY]
```

List all disks.

```shell
ark_kv list
```

### read key

```shell
ark_kv read key DISK KEY SECRET_KEY [--file FILE]
```

Read key `world.txt` in disk `Hello` using secret key `./secret.key` and write to `stdout`.

```shell
ark_kv read key Hello world.txt ./secret.key
```

Read key `slurm.txt` in disk `Super` using secret key `./secret.key` and write to file `./slurm.txt`.

```shell
ark_kv read key Hello world.txt ./secret.key --file slurm.txt
```

### write key

```shell
ark_kv write key DISK KEY [--file FILE] [--str STR]
```

Write key `slurm.txt` in disk `Super` from `stdin`. Use `CTRL+D` to close input.

```shell
ark_kv write key Super slurm.txt
```

Write key `world.txt` in disk `Hello` from string `"Hello, world!"`.

```shell
ark_kv write key Hello world.txt --str "Hello, world!"
```

Write key `cake.txt` in disk `Triumph` from file `./cake.txt`.

```shell
ark_kv write key Triumph cake.txt --file ./cake.txt
```
