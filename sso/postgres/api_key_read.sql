SELECT
    "created_at",
    "updated_at",
    "id",
    "client_id",
    "user_id",
    "name",
    "enable",
    "scope"
FROM
    sso.api_key_table
WHERE
    "client_id" = $1
AND
    CASE WHEN ($2::UUID[] IS NULL) THEN
        TRUE
    ELSE
        "id" = ANY($2)
    END
AND
    CASE WHEN ($3::UUID[] IS NULL) THEN
        TRUE
    ELSE
        "user_id" = ANY($3)
    END
ORDER BY
    "name" ASC
