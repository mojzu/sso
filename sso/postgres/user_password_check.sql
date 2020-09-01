WITH cte AS (
    SELECT
        u."id",
        up."hash",
        u."enable",
        up."require_update"
    FROM
        sso.user_table AS u
    INNER JOIN
        sso.user_password_table AS up ON up."user_id" = u."id"
    WHERE
        u."email" = $1
)
SELECT
    "id",
    CASE WHEN ("hash" IS NULL) THEN
        FALSE
    ELSE
        sso._password_check($2, "hash")
    END AS "check",
    "enable",
    "require_update"
FROM
    cte
