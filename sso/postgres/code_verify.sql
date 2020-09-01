WITH ttl AS (
    DELETE FROM
        sso.code_table
    WHERE
        "ttl" < CURRENT_TIMESTAMP
)
DELETE FROM
    sso.code_table
WHERE
    "client_id" = $1
AND
    "value" = $2
AND
    "target" = $3
RETURNING
    "client_id",
    "user_id",
    "state",
    "scope",
    "email"
