SELECT
    "type",
    COALESCE("status_code", 0::SMALLINT) AS "status_code",
    COUNT(*) as "count"
FROM sso.audit_table
WHERE
    CASE
        WHEN $1 IS NULL THEN TRUE
        ELSE "created_at" >= $1
    END
AND
    CASE
        WHEN $2 IS NULL THEN TRUE
        ELSE "client_id" = $2
    END
GROUP BY
    "type",
    "status_code"
ORDER BY
    "type" ASC
