FROM timescale/timescaledb:latest-pg11
ADD 000_create.sh /docker-entrypoint-initdb.d/
ADD pg_restore.sh /opt/
ADD sso.pgdump /opt/
