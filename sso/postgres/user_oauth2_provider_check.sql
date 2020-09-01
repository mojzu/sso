SELECT
    "user_id"
FROM
    sso.user_oauth2_provider_table
WHERE
    "oauth2_provider" = $1
AND
    "sub" = $2
