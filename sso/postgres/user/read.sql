WITH cte AS (
    SELECT
        "user_id",
        COUNT(*) as "count"
    FROM
        sso.user_oauth2_provider_table
    GROUP BY
        "user_id"
)
SELECT
    u."created_at",
    u."updated_at",
    u."id",
    u."name",
    u."email",
    u."locale",
    u."timezone",
    u."enable",
    u."static",
    up."created_at" as "password_created_at",
    up."updated_at" as "password_updated_at",
    up."user_id" as "password_user_id",
    up."allow_reset" as "password_allow_reset",
    up."require_update" as "password_require_update",
    up."static" as "password_static",
    a."created_at" as "access_created_at",
    a."updated_at" as "access_updated_at",
    a."user_id" as "access_user_id",
    a."client_id" as "access_client_id",
    a."enable" as "access_enable",
    a."scope" as "access_scope",
    a."static" as "access_static",
    COALESCE(c."count", 0) as "oauth2_provider_count"
FROM
    sso.user_table AS u
LEFT JOIN
    sso.user_password_table AS up ON up."user_id" = u."id"
LEFT JOIN
    sso.access_table AS a ON a."client_id" = $1::UUID AND a."user_id" = u."id"
LEFT JOIN
    cte AS c ON c."user_id" = u."id"
WHERE
    CASE WHEN ($2::UUID[] IS NULL) THEN
        TRUE
    ELSE
        "id" = ANY($2)
    END
AND
    CASE WHEN ($3::VARCHAR[] IS NULL) THEN
        TRUE
    ELSE
        "email" = ANY($3)
    END
ORDER BY
    "name" ASC
