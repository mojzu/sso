INSERT INTO
    sso.token_table("id", "client_id", "user_id", "ttl", "value", "value_refresh", "name", "enable", "scope")
VALUES
    ($1, $2, $3, sso._ttl_timestamptz($4), sso._secret_generate(32), sso._secret_generate(32), $5, $6, $7)
RETURNING
    sso._secret_encrypt($8, "value", "id"::VARCHAR) as "value",
    sso._secret_encrypt($8, "value_refresh", "id"::VARCHAR) as "value_refresh",
    "scope"
