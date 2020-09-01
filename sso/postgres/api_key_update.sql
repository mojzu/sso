UPDATE
    sso.api_key_table
SET
    "name" = COALESCE($3, "name"),
    "enable" = COALESCE($4, "enable")
WHERE
    "client_id" = $1
AND
    "id" = $2
RETURNING
    "created_at",
    "updated_at",
    "id",
    "client_id",
    "user_id",
    "name",
    "enable",
    "scope"
