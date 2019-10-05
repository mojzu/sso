# data

Encrypted key, value storage library and binary crate.

- **Asymmetric encryption**, key value data is encrypted, reads require a secret key.
- **Compression**, key value data is compressed to reduce disk use.
- **Integrity**, key value data is hashed on write, hash is checked on read.
- **Versioning**, key value writes are versioned.
- **Retention**, key value version retention is configurable (number of versions and/or duration of time).

## CLI

### create secret-key

TODO(refactor): Binary `data`.

```shell
data create secret-key <SECRET_KEY>
```

Create secret key and write to file `./secret.key`.

```shell
data create secret-key ./secret.key
```

### verify secret-key

```shell
data verify secret-key <SECRET_KEY>
```

Verify secret key read from file `./secret.key`.

```shell
data verify secret-key ./secret.key
```

### verify key

```shell
data verify key [OPTIONS] <DISK> <KEY> <SECRET_KEY>
```

### create disk

```shell
data create disk [OPTIONS] <DISK> <SECRET_KEY>
```

Create a new disk named `Foo` with secret key read from file `./secret.key`.

```shell
data create disk Foo ./secret.key
```

Create a new disk named `Bar` with secret key read from file `./secret.key`, retain at least 10 key versions, retain all key versions for a duration of at least 1 week (604800 seconds).

```shell
data create disk Bar ./secret.key  --version-retention 10 --duration-retention 604800
```

### list

```shell
data list [ARGS]
```

List all disks.

```shell
data list
```

List all keys in disk `Foo`.

```shell
data list Foo
```

List versions of key `bar.txt` in disk `Foo`.

```shell
data list Foo bar.txt
```

### status

```shell
data status [ARGS]
```

Get status of all disks.

```shell
data status
```

Get status of disk `Import`.

```shell
data status Import
```

Get status of key `blns.txt` in disk `Import`.

```shell
data status Import blns.txt
```

### read key

```shell
data read key [OPTIONS] <DISK> <KEY> <SECRET_KEY>
```

Read key `world.txt` in disk `Hello` using secret key `./secret.key` and write to `stdout`.

```shell
data read key Hello world.txt ./secret.key
```

Read key `slurm.txt` in disk `Super` using secret key `./secret.key` and write to file `./slurm.txt`.

```shell
data read key Hello world.txt ./secret.key --file slurm.txt
```

### read disk

```shell
data read disk [OPTIONS] <DISK> <SECRET_KEY>
```

Read keys in disk `Import` into directory `./tests/file2` using secret key `./secret.key`.

```shell
data read disk Import ./secret.key --directory ./tests/file2
```

### write key

```shell
data write key [OPTIONS] <DISK> <KEY>
```

Write key `slurm.txt` in disk `Super` from `stdin`. Use `CTRL+D` to close input.

```shell
data write key Super slurm.txt
```

Write key `world.txt` in disk `Hello` from string `"Hello, world!"`.

```shell
data write key Hello world.txt --str "Hello, world!"
```

Write key `cake.txt` in disk `Triumph` from file `./cake.txt`.

```shell
data write key Triumph cake.txt --file ./cake.txt
```

### write disk

```shell
data write disk [OPTIONS] <DISK>
```

Write keys in disk `Import` from directory `./tests/file`.

```shell
data write disk Import --directory ./tests/file
```

### delete

```shell
data delete <DISK> [KEY]
```

Delete disk `Combine`.

```shell
data delete Combine
```

Delete key `escape.txt` in disk `Ape`.

```shell
data delete Ape escape.txt
```

### poll

```shell
data poll [FLAGS]
```

Poll disks and vacuum database.

```shell
data poll --vacuum
```

### mount

```shell
data mount <DISK> <MOUNTPOINT>
```

Mount disk `Documents` at path `/tmp/documents`.

```shell
mkdir -p /tmp/documents
data mount Documents /tmp/documents
fusermount -u /tmp/documents
```

## Test

```shell
# Backup.
data create disk archive ~/secret.key
data create disk documents ~/secret.key --version-retention 10 --duration-retention 2419200
data write disk archive --directory /home/$USER/archive
data write disk documents --directory /home/$USER/documents
data poll --vacuum

# Restore.
data read disk archive ~/secret.key --directory /home/$USER/archive2
diff -r /home/$USER/archive /home/$USER/archive2
data read disk documents ~/secret.key --directory /home/$USER/documents2
diff -r /home/$USER/documents /home/$USER/documents2
```

## References

- <https://en.wikipedia.org/wiki/Directed_acyclic_graph>
- <https://github.com/zerotier/lf>
