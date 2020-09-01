INSERT INTO
    sso.api_key_table("id", "client_id", "user_id", "value", "name", "enable", "scope")
VALUES
    ($1, $2, $3, $4, $5, $6, $7)
RETURNING
    "created_at",
    "updated_at",
    "id",
    "client_id",
    "user_id",
    "name",
    "enable",
    "scope"
