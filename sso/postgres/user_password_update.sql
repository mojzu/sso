WITH cte AS (
    SELECT
        "user_id"
    FROM
        sso.user_password_table
    WHERE
        "user_id" = $1
    AND
        sso._password_check($2, "hash")
)
UPDATE
    sso.user_password_table AS up
SET
    "hash" = sso._password_hash($3)
FROM
    cte
WHERE
    cte."user_id" = up."user_id"
AND
    "static" IS FALSE
