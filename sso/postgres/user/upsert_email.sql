INSERT INTO
    sso.user_table("id", "name", "email")
VALUES
    ($1, $2, $3)
ON CONFLICT
    ("email")
DO UPDATE SET
    "name" = $2,
    "email" = $3
RETURNING
    "id"
