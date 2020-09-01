INSERT INTO
    sso.code_table("client_id", "value", "target", "ttl", "email")
SELECT
    $1,
    sso._secret_generate(32),
    'Register',
    sso._ttl_timestamptz($2),
    $3
RETURNING
    "value"
