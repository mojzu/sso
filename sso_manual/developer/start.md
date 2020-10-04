# Start

Clone repository and `Open folder in container`, using the vscode [Remote Development](https://code.visualstudio.com/docs/remote/containers) extension.

Run vscode [Tasks](https://code.visualstudio.com/docs/editor/tasks) using the command palette or the Task Explorer extension.

## Configuring

The `sso_server` and `sso_cli` binaries depend on a configuration file, by default this is read from `./.config/sso.toml` or it can be configured with the `--config` command line argument (For example `--config .config/sso.example`). An example configuration file can be found at `.config/sso.example.toml`.

OAuth2 clients and users can be statically defined in this file, configuration can be generated using the `sso_cli` binary. A postgres database connection is required, the example configuration works with the postgres service started by the [postgres - up](tasks.md#postgres-up) task.

Run the [cargo - install](tasks.md#cargo-install) task.

To generate a client configuration run the following command, save the client secret key and copy the TOML output into the configuration file.

```bash
sso_cli generate client $NAME $REDIRECT_URI
```

to generate a user configuration run the following command, save the user password and copy the TOML output into the configuration file.

```bash
sso_cli generate user $NAME $EMAIL
```

## Developing

Run the following tasks to compile and run tests during development, tests depend on [Webdriver Manager](#webdriver-manager).

-   [postgres - up](tasks.md#postgres-up)
-   [cargo - sso_server](tasks.md#cargo-sso_server)
-   [npm - start client example (express, passport)](tasks.md#npm-start-client-example-express-passport)
-   [npm - run host tests](tasks.md#npm-run-host-tests)

To stop and destroy postgres services.

-   [postgres - down](tasks.md#postgres-down)

## Testing

Run the following tasks to build docker images and run tests for production, tests depend on [Webdriver Manager](#webdriver-manager).

-   [docker test - build](tasks.md#docker-test-build)
-   [docker test - up](tasks.md#docker-test-up)
-   [docker test - protractor](tasks.md#docker-test-protractor)

To stop and destroy test services.

-   [docker test - down](tasks.md#docker-test-down)

Commands to run test CI locally can be found in `.github/workflows/docker.yml`.

## Documenting

Run the following tasks to serve the manual development server or build manual static site.

-   [mkdocs - serve](tasks.md#mkdocs-serve)
-   [mkdocs - build](tasks.md#mkdocs-build)

## Client Generation

The crate `sso_client` is generated using [paperclip](https://github.com/wafflespeanut/paperclip), install it in the development container with the following command.

```bash
cargo install paperclip --git https://github.com/wafflespeanut/paperclip --features cli
```

When `sso_server` is running, run the following tasks to download a copy of the OpenAPI specification file and generate the client.

-   [openapi - wget openapi](tasks.md#openapi-wget-openapi)
-   [openapi - generate rust client](tasks.md#openapi-generate-rust-client)
-   [openapi - generate typescript client](tasks.md#openapi-generate-typescript-client)

fix: File requires manual changes, adding `"type": "object"` to definitions, or paperclip will not generate the expected files.

## Designing

Client HTML templates change the appearance of the web interface, run the template builder using the `npm - run template builder` task.

Template source files are located in the `sso_test/template` directory. A build will write output files to the `sso_test/tmp/template` directory. Open `index.html` in a browser to check the appearance, and add `template.html` to the configuration file when complete.

## Labels

Some labels are used in comments throughout the code which can be searched for.

-   `todo: ...`: Stuff to work on
-   `fix: ...`: Notes on how something has been fixed
-   `depend: ...`: Dependencies that should be upgraded occasionally
-   `test: ...`: Notes related to testing

For example.

```bash
# depend: package.json
# run task `npm - update packages`
# depend: Cargo.toml
# run task `cargo - update`
# fix: autoprefixer must be < 10 or postcss will error
```

## Webdriver Manager

Tests depend on [Webdriver Manager](https://www.npmjs.com/package/webdriver-manager), start it outside of the remote development container using the commands.

```bash
npm run webdriver-update
npm run webdriver-start
```

## pgAdmin

Available at [localhost:8000](http://localhost:8000) when postgres services are running.

-   Login with username `guest` and password `guest`
-   Select `Add New Server`
    -   General - Name: `postgres`
    -   Connection - Host name/address: `postgres`
    -   Connection - Port: `5432`
    -   Connection - Maintenance database: `postgres`
    -   Connection - Username: `postgres`
    -   Connection - Password: `postgres`
    -   Connection - Save password: `yes`
-   Save and select `postgres` from server list

## PgHero

Available at [localhost:8001](http://localhost:8001) when postgres services are running.
