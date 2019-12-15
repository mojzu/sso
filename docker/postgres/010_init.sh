#!/bin/bash
set -e

# Create database.
psql -v ON_ERROR_STOP=1 -U "$POSTGRES_USER" <<-EOSQL
    CREATE DATABASE sso;
    GRANT ALL PRIVILEGES ON DATABASE sso TO "$POSTGRES_USER";
EOSQL

# Restore .pgdump file if it exists and is not empty.
if [ -s "/pgdump/sso.pgdump" ]
then
    pg_restore -U "$POSTGRES_USER" -d sso /pgdump/sso.pgdump
fi
