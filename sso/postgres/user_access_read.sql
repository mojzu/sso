SELECT
    "created_at",
    "updated_at",
    "client_id",
    "user_id",
    "enable",
    "scope",
    "static"
FROM
    sso.access_table
WHERE
    "client_id" = $1
AND
    "user_id" = $2
