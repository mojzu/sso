# Tasks

## all - clean

Removes build artifacts.

## all - distclean

Removes build artifacts and installed dependencies and their caches.

## cargo - install

Installs `sso_server` and `sso_cli` into development container, which lets you run `sso_cli ...` instead of `cargo run --bin sso_cli -- ...` in the terminal.

## cargo - sso_server

Build and start `sso_server` using configuration file `.config/sso.toml`, available at `http://localhost:7042/`.

## docker test - build

Build docker images in `.devcontainer/docker-test.yml`.

## docker test - up

Build and start docker compose services in `.devcontainer/docker-test.yml`.

## docker test - down

Stop and destroy docker compose services in `.devcontainer/docker-test.yml`.

## docker test - protractor

Run [Protractor](https://www.protractortest.org/) tests against docker compose services in `.devcontainer/docker-test.yml`.

## mkdocs - serve

Run [mkdocs](https://www.mkdocs.org/) live development server, available at [localhost:8079](http://localhost:8079).

## mkdocs - build

Build [mkdocs](https://www.mkdocs.org/) static site output in `docs` directory for github pages.

## npm - run template builder

Build template files from sources in `sso_test/template`. The output file `sso_test/tmp/template/index.html` can be opened in a browser to check page styles, and `sso_test/tmp/template/template.html` can be added to the `sso` configuration file.

## npm - start client example (express, passport)

Build and start client example application, available at [localhost:8080](http://localhost:8080).

## npm - run host tests

Run tests in `sso_test/test/host.ts` file.

## openapi - wget openapi

Download a copy of `openapi.json` file from server and save to `sso_manual/openapi.json`. This file is required for client generation tasks.

## openapi - generate typescript client

Generate TypeScript client using [OpenAPI Generator](https://github.com/OpenAPITools/openapi-generator).

## openapi - generate rust client

Generate rust client using [paperclip](https://github.com/wafflespeanut/paperclip).

## postgres - build

Build docker images in `.devcontainer/docker-postgres.yml`.

## postgres - up

Build and start docker compose services in `.devcontainer/docker-postgres.yml`.

Connection string is `postgres://postgres:postgres@localhost:5432/postgres`.

Services include [pgAdmin](start.md#pgadmin) and [PgHero](start.md#pghero).

## postgres - down

Stop and destroy docker compose services in `.devcontainer/docker-postgres.yml`.
