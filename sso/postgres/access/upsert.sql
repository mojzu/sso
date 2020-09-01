INSERT INTO
    sso.access_table("client_id", "user_id", "enable", "scope", "static")
VALUES
    ($1, $2, $3, $4, $5)
ON CONFLICT
    ("client_id", "user_id")
DO UPDATE SET
    "enable" = $3,
    "scope" = $4,
    "static" = $5
RETURNING
    "created_at",
    "updated_at",
    "client_id",
    "user_id",
    "enable",
    "scope",
    "static"
