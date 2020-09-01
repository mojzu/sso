DELETE FROM
    sso.access_table
WHERE
    "client_id" = $1
AND
    "user_id" = $2
