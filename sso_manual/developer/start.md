# Start

Clone repository and `Open folder in container`, using the vscode [Remote Development](https://code.visualstudio.com/docs/remote/containers) extension.

Run vscode [Tasks](https://code.visualstudio.com/docs/editor/tasks) using the command palette or the Task Explorer extension.

## Developing

Run the following commands to compile and run tests during development, tests depend on [Webdriver Manager](#webdriver-manager).

-   [postgres - build](tasks.md#postgres-build)
-   [postgres - up](tasks.md#postgres-up)
-   [cargo - sso_server](tasks.md#cargo-sso_server)
-   [npm - start client example (express, passport)](tasks.md#npm-start-client-example-express-passport)
-   [npm - run host tests](tasks.md#npm-run-host-tests)

To stop and destroy postgres services.

-   [postgres - down](tasks.md#postgres-down)

## Testing

Run the following commands to build docker images and run tests for production, tests depend on [Webdriver Manager](#webdriver-manager).

-   [docker test - build](tasks.md#docker-test-build)
-   [docker test - up](tasks.md#docker-test-up)
-   [docker test - protractor](tasks.md#docker-test-protractor)

To stop and destroy test services.

-   [docker test - down](tasks.md#docker-test-down)

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
