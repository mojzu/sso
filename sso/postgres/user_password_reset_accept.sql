UPDATE
    sso.user_password_table
SET
    "hash" = sso._password_hash($2)
WHERE
    "user_id" = $1
AND
    "static" IS FALSE
