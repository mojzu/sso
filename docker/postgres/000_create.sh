#!/bin/bash
set -e

# Create databases.
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
	CREATE DATABASE sso;
	GRANT ALL PRIVILEGES ON DATABASE sso TO "$POSTGRES_USER";
EOSQL
