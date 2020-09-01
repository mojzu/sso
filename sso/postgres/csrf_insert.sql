INSERT INTO
    sso.csrf_table("client_id", "token", "ttl")
VALUES
    ($1, sso._secret_generate(32), sso._ttl_timestamptz($2))
RETURNING
    "created_at",
    "client_id",
    "token",
    "ttl"
