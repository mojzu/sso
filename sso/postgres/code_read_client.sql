WITH ttl AS (
    DELETE FROM
        sso.code_table
    WHERE
        "ttl" < CURRENT_TIMESTAMP
)
SELECT
    "client_id"
FROM
    sso.code_table
WHERE
    "value" = $1
