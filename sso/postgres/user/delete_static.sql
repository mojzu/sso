DELETE FROM
    sso.user_table
WHERE
    "static" IS TRUE
AND
    "id" != ALL($1)
