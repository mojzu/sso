# <https://docs.timescale.com/latest/using-timescaledb/backup>
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" sso -c "SELECT timescaledb_pre_restore();"
pg_restore -U guest -d sso /opt/sso.pgdump
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" sso -c "SELECT timescaledb_post_restore();"
