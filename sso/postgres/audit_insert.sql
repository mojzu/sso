INSERT INTO
    sso.audit_table("client_id", "user_id", "token_id", "api_key_id", "type", "subject", "data", "status_code")
VALUES
    ($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING
    "created_at",
    "id",
    "client_id",
    "user_id",
    "token_id",
    "api_key_id",
    "type",
    "subject",
    "data",
    "status_code"
