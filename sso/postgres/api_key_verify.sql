SELECT
    "created_at",
    "updated_at",
    "id",
    "client_id",
    "user_id",
    "name",
    "enable",
    "scope"
FROM
    sso.api_key_table
WHERE
    "client_id" = $1::UUID
AND
    "id" = $2::UUID
AND
    sso._secret_check($3::VARCHAR, "value"::VARCHAR, "id"::VARCHAR)
