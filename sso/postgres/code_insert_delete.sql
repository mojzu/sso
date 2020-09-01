WITH cte AS (
    SELECT
        u."id",
        u."email"
    FROM
        sso.user_password_table AS up
    INNER JOIN
        sso.user_table AS u ON u."id" = up."user_id"
    WHERE
        up."user_id" = $3
    AND
        sso._password_check($4, "hash")
)
INSERT INTO
    sso.code_table("client_id", "value", "target", "ttl", "user_id", "email")
SELECT
    $1,
    sso._secret_generate(32),
    'Delete',
    sso._ttl_timestamptz($2),
    cte."id",
    cte."email"
FROM
    cte
RETURNING
    "value",
    "email"
