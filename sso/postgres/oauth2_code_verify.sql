WITH ttl AS (
    DELETE FROM
        sso.oauth2_code_table
    WHERE
        "ttl" < CURRENT_TIMESTAMP
)
DELETE FROM
    sso.oauth2_code_table
WHERE
    "client_id" = $1
AND
    "csrf" = $2
RETURNING
    "provider",
    "target",
    "pkce",
    "redirect_uri",
    "state",
    "scope",
    "email"
