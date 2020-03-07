SELECT
  a."type" AS "type_",
  COALESCE(a.status_code, 0 :: SMALLINT) AS "status_code",
  count(*) AS "count"
FROM sso_audit AS a
WHERE
  a."created_at" > $1
  AND CASE
    WHEN $2 IS NULL THEN TRUE
    ELSE a."service_id" = $2
  END
GROUP BY
  a."type",
  a."status_code"
ORDER BY
  a."type" ASC;
