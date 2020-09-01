UPDATE
    sso.user_table AS u
SET
    "name" = COALESCE($2, "name"),
    "email" = COALESCE($3, "email"),
    "locale" = COALESCE($4, "locale"),
    "timezone" = COALESCE($5, "timezone"),
    "enable" = COALESCE($6, "enable")
WHERE
    "id" = $1
AND
    "static" IS FALSE
