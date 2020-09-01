WITH ttl AS (
    DELETE FROM
        sso.csrf_table
    WHERE
        "ttl" < CURRENT_TIMESTAMP
)
DELETE FROM
    sso.csrf_table
WHERE
    "client_id" = $1
AND
    "token" = $2
RETURNING
    "client_id",
    "token"
