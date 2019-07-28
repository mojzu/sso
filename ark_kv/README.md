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
ark_kv create secret-key <SECRET_KEY>
```

Create secret key and write to file `./secret.key`.

```shell
ark_kv create secret-key ./secret.key
```

### verify secret-key

```shell
ark_kv verify secret-key <SECRET_KEY>
```

Verify ark key read from file `./secret.key`.

```shell
ark_kv verify secret-key ./secret.key
```

### verify key

```shell
ark_kv verify key [OPTIONS] <DISK> <KEY> <SECRET_KEY>
```

### create disk

```shell
ark_kv create disk [OPTIONS] <DISK> <SECRET_KEY>
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
ark_kv list [ARGS]
```

List all disks.

```shell
ark_kv list
```

List all keys in disk `Foo`.

```shell
ark_kv list Foo
```

List versions of key `bar.txt` in disk `Foo`.

```shell
ark_kv list Foo bar.txt
```

### status

```shell
ark_kv status [ARGS]
```

Get status of all disks.

```shell
ark_kv status
```

Get status of disk `Import`.

```shell
ark_kv status Import
```

Get status of key `blns.txt` in disk `Import`.

```shell
ark_kv status Import blns.txt
```

### read key

```shell
ark_kv read key [OPTIONS] <DISK> <KEY> <SECRET_KEY>
```

Read key `world.txt` in disk `Hello` using secret key `./secret.key` and write to `stdout`.

```shell
ark_kv read key Hello world.txt ./secret.key
```

Read key `slurm.txt` in disk `Super` using secret key `./secret.key` and write to file `./slurm.txt`.

```shell
ark_kv read key Hello world.txt ./secret.key --file slurm.txt
```

### read disk

```shell
ark_kv read disk [OPTIONS] <DISK> <SECRET_KEY>
```

Read keys in disk `Import` into directory `./tests/file2` using secret key `./secret.key`.

```shell
ark_kv read disk Import ./secret.key --directory ./tests/file2
```

### write key

```shell
ark_kv write key [OPTIONS] <DISK> <KEY>
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

### write disk

```shell
ark_kv write disk [OPTIONS] <DISK>
```

Write keys in disk `Import` from directory `./tests/file`.

```shell
ark_kv write disk Import --directory ./tests/file
```

### delete

```shell
ark_kv delete <DISK> [KEY]
```

Delete disk `Combine`.

```shell
ark_kv delete Combine
```

Delete key `escape.txt` in disk `Ape`.

```shell
ark_kv delete Ape escape.txt
```

### poll

```shell
ark_kv poll [FLAGS]
```

Poll disks and vacuum database.

```shell
ark_kv poll --vacuum
```

### mount

```shell
ark_kv mount <DISK> <MOUNTPOINT>
```

Mount disk `Documents` at path `/tmp/documents`.

```shell
mkdir -p /tmp/documents
ark_kv mount Documents /tmp/documents
fusermount -u /tmp/documents
```
