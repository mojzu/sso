UPDATE sso_audit
    SET "updated_at" = $2,
        "status_code" = COALESCE("status_code", $3),
        "subject" = COALESCE("subject", $4),
        "data" = $5 || "data"
    WHERE "id" = $1 AND CASE
        WHEN $6 IS NULL THEN TRUE
        ELSE "service_id" = $6
    END
    RETURNING "created_at", "updated_at", "id", "user_agent", "remote", "forwarded", "status_code",
        "type" AS "type_", "subject", "data", "key_id", "service_id", "user_id", "user_key_id";
