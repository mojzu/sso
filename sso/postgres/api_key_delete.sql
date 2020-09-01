DELETE FROM
    sso.api_key_table
WHERE
    "client_id" = $1
AND
    "id" = $2
