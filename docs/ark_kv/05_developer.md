# Developer

## Test

To run unit tests.

```shell
cargo make test
```

For integration tests, the following environment variables are required.

| Variable        | Description  |
| --------------- | ------------ |
| TEST_ARK_KV_BIN | Binary path. |

To run integration tests.

```shell
cargo make test-integration
```
