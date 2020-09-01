INSERT INTO
    sso.oauth2_code_table("client_id", "csrf", "provider", "target", "ttl", "pkce", "email")
VALUES
    ($1, $2, $3, $4, sso._ttl_timestamptz($5), $6, $7)
RETURNING
    "csrf"
