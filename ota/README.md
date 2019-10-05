# ota

Over-the-air updates library and binary crate.

TODO(feature): Implement/document this.

## Notes

-   Binary release ($name-$version-$target.tar.xz)

manifest.json

```json
{
    "data": [
        {
            "name": "sso",
            "version": "0.1.0",
            "target": "x86_64-unknown-linux-gnu",
            "url": "https://test.com/update/sso-0.1.0-x86_64-unknown-linux-gnu.tar.xz",
            "hash": "sha512:asdk5634jk6b3jk4b6346..."
        },
        ...
    ]
}
```

bootloader

- build `ota` using name, target, manifest url and keys
- provision directory
    - /ota
    - /ota.lock
    - /v1/sso
    - /v2/...
- author releases using binary, name, version, target, manifest url and keys

### Environment

ota build
    --bin RELEASE_BINARY
    --name RELEASE_NAME
    --version RELEASE_VERSION
    --target RELEASE_TARGET
    --author-key RELEASE_AUTHOR_KEY
    [--manifest MANIFEST_URL/FILE]
    --output RELEASE_DIRECTORY

- sha512 hash of binary
- encrypt binary using author key (+sign?)
- compress encrypted binary and write to output directory
- create manifest entry using name, version, target, hash, url/file link (+datetime?)
- sign manifest entry using author key
- load existing manifest if provided and append, write new/updated manifest file to output directory
- space for custom key/value data in manifest?

- post url for ota status updates (e.g. localhost:4242/v1/ota)
- ota start|update|revert [--auto-update]
- keys built into binary
- version increment configuration, update time configuration, scripts to check compatability, instructions?
- validate signatures, check date/times for age to prevent rollback, check binary types/compression etc.
- hash files and verify hashes on download/use.
- version comparison, version code?
- embedded manifest location(s), configurable provision directory (environment variable?)
- verify name, target, version, other conditions for installation

```shell
ota start --auto-update
# ota reads + verifies lock file (lock file is signed and/or encrypted?)
# ota verifies current version using lock file information
# ota starts process of current version (e.g. /v1/sso)
# ota polls + verifies manifest file for new versions (url|local file support)
# ota downloads, decrypts and verifies update, writes to next version (e.g. /v2/sso)
# ota writes provisional change to lock file, updating current version
# ota stops running process, starts current version
# ota commits change to lock file if process runs for X time or exits with code 0
# ota reverts change to lock file if process exits with error code, starts process of previous version
```

- <https://tools.ietf.org/html/draft-ietf-suit-architecture-00>
