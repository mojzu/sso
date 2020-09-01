#!/bin/sh
# Adapted from PostGIS init script
# <https://raw.githubusercontent.com/postgis/docker-postgis/master/12-3.0/initdb-postgis.sh>
set -e

# Perform all actions as $POSTGRES_USER
export PGUSER="$POSTGRES_USER"

# Run pgTAP self tests
# Disabled due to test/build, test/sql permission denied errors
# export PGDATABASE="$POSTGRES_DB"
# (cd /usr/share/postgresql/12/contrib/pgtap-1.1.0 \
#     && make installcheck \
#     && make test)

# Create the 'template_pgtap' template db
"${psql[@]}" <<- 'EOSQL'
CREATE DATABASE template_pgtap IS_TEMPLATE true;
EOSQL

# Load pgTAP into both template_database and $POSTGRES_DB
for DB in template_pgtap "$POSTGRES_DB"; do
        echo "Loading pgTAP extensions into $DB"
        "${psql[@]}" --dbname="$DB" <<-'EOSQL'
                CREATE EXTENSION IF NOT EXISTS pgtap;
                CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
EOSQL
done
