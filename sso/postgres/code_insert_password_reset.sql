WITH cte AS (
    SELECT
        "user_id"
    FROM
        sso.user_password_table AS up
    INNER JOIN
        sso.user_table AS u ON u."id" = up."user_id"
    WHERE
        u."email" = $3
)
INSERT INTO
    sso.code_table("client_id", "value", "target", "ttl", "user_id")
SELECT
    $1,
    sso._secret_generate(32),
    'PasswordReset',
    sso._ttl_timestamptz($2),
    cte."user_id"
FROM
    cte
RETURNING
    "value"
