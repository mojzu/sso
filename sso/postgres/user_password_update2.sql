UPDATE
    sso.user_password_table
SET
    "allow_reset" = COALESCE($2, "allow_reset"),
    "require_update" = COALESCE($3, "require_update")
WHERE
    "user_id" = $1
AND
    "static" IS FALSE
RETURNING
    "created_at",
    "updated_at",
    "user_id",
    "allow_reset",
    "require_update",
    "static"
