INSERT INTO
    sso.user_table("id", "name", "email", "locale", "timezone", "enable")
VALUES
    ($1, $2, $3, $4, $5, $6)
RETURNING
    "created_at",
    "updated_at",
    "id",
    "name",
    "email",
    "locale",
    "timezone",
    "enable",
    "static"
