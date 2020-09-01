INSERT INTO
    sso.user_table("id", "name", "email", "locale", "timezone", "enable", "static")
VALUES
    ($1, $2, $3, $4, $5, $6, TRUE)
ON CONFLICT
    ("id")
DO UPDATE SET
    "name" = $2,
    "email" = $3,
    "locale" = $4,
    "timezone" = $5,
    "enable" = $6,
    "static" = TRUE
