SELECT
    "client_id",
    "user_id",
    "created_at",
    "updated_at",
    "enable",
    "scope",
    "static"
FROM
    sso.access_table
WHERE
    "client_id" = $1
