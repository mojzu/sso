INSERT INTO
    sso.user_password_table("user_id", "hash", "allow_reset", "require_update", "static")
VALUES
    ($1, $2, $3, $4, $5)
ON CONFLICT
    ("user_id")
DO UPDATE SET
    "hash" = $2,
    "allow_reset" = $3,
    "require_update" = $4,
    "static" = $5
RETURNING
    "created_at",
    "updated_at",
    "user_id",
    "allow_reset",
    "require_update",
    "static"
