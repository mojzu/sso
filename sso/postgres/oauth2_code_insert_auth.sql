INSERT INTO
    sso.oauth2_code_table("client_id", "csrf", "provider", "target", "ttl", "pkce", "redirect_uri", "state", "scope")
VALUES
    ($1, $2, $3, $4, sso._ttl_timestamptz($5), $6, $7, $8, $9)
RETURNING
    "csrf"
