UPDATE
    sso.access_table
SET
    "enable" = COALESCE($3, "enable"),
    "scope" = COALESCE($4, "scope")
WHERE
    "client_id" = $1
AND
    "user_id" = $2
AND
    "static" IS FALSE
RETURNING
    "created_at",
    "updated_at",
    "client_id",
    "user_id",
    "enable",
    "scope",
    "static"
