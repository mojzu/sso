WITH ttl AS (
    DELETE FROM
        sso.token_table
    WHERE
        ("ttl" + ($2::BIGINT * '1s'::INTERVAL)) < CURRENT_TIMESTAMP
)
SELECT
    token."scope" as "scope",
    u."name" as "username",
    u."id"::VARCHAR as "sub"
FROM
    sso._secret_decrypt($3, $4) AS sd
INNER JOIN
    sso.token_table AS token ON token."id" = sd."value"::UUID
INNER JOIN
    sso.user_table AS u ON u."id" = token."user_id"
WHERE
    sso._secret_check(token."value", sd."hash", sd."value")
AND
    token."client_id" = $1
AND
    token."ttl" > CURRENT_TIMESTAMP
