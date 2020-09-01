WITH ttl AS (
    DELETE FROM
        sso.token_table
    WHERE
        ("ttl" + ($3::BIGINT * '1s'::INTERVAL)) < CURRENT_TIMESTAMP
),
cte AS (
    SELECT
        token."id"
    FROM
        sso._secret_decrypt($4, $5) AS sd
    INNER JOIN
        sso.token_table AS token ON token."id" = sd."value"::UUID
    WHERE
        sso._secret_check(token."value_refresh", sd."hash", sd."value")
    AND
        token."client_id" = $1
)
UPDATE
    sso.token_table AS token
SET
    "ttl" = sso._ttl_timestamptz($2::BIGINT),
    "value" = sso._secret_generate(32),
    "value_refresh" = sso._secret_generate(32)
FROM
    cte
WHERE
    cte."id" = token."id"
AND
    token."client_id" = $1
RETURNING
    sso._secret_encrypt($4, token."value", token."id"::VARCHAR) as "value",
    sso._secret_encrypt($4, token."value_refresh", token."id"::VARCHAR) as "value_refresh",
    token."scope" as "scope"
