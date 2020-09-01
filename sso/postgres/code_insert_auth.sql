INSERT INTO
    sso.code_table("client_id", "value", "target", "ttl", "user_id", "state", "scope")
VALUES
    ($1, sso._secret_generate(32), 'Auth', sso._ttl_timestamptz($2), $3, $4, $5)
RETURNING
    "value"
