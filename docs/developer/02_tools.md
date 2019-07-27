# Tools

The ark_auth crate depends on [PostgreSQL][postgresql] and [SQLite][sqlite] libraries, to install them on Debian Linux distributions.

```shell
sudo apt install libpq-dev libsqlite3-dev libssl-dev libfuse-dev pkg-config
```

Install [Rust][rust] using [rustup][rustup]. Check required components are installed..

```shell
rustup component add rustfmt
rustup component add clippy
```

The ark_auth crate depends on [Cargo Make](cargo-make) and [Diesel](diesel), install them with Cargo.

```shell
cargo install --force cargo-make
cargo install --force diesel_cli --no-default-features --features "postgres sqlite"
```

To update the toolchain.

```shell
rustup self update
rustup update
```

To update crate dependencies.

```shell
cargo update
```

[Docker][docker] and [Docker Compose][docker-compose] are used for development, install them using the linked documentation.

To start containers defined in `docker-compose.yml`.

```shell
docker-compose up
```

To stop containers.

```shell
docker-compose down
```

[postgresql]: <https://www.postgresql.org/>
[sqlite]: <https://www.sqlite.org/index.html>
[rust]: <https://www.rust-lang.org/>
[rustup]: <https://rustup.rs/>
[cargo-make]: <https://github.com/sagiegurari/cargo-make>
[diesel]: <http://diesel.rs/>
[docker]: <https://docs.docker.com/install/>
[docker-compose]: <https://docs.docker.com/compose/>
