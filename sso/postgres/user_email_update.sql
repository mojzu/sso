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
    sso.user_table AS u
SET
    "email" = $3
FROM
    cte
WHERE
    cte."user_id" = u."id"
AND
    u."static" IS FALSE
