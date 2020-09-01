SELECT
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
FROM
    sso.audit_table
WHERE
    "id" = $1
