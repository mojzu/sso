WITH ttl AS (
    DELETE FROM
        sso.oauth2_code_table
    WHERE
        "ttl" < CURRENT_TIMESTAMP
)
SELECT
    "client_id"
FROM
    sso.oauth2_code_table
WHERE
    "csrf" = $1
