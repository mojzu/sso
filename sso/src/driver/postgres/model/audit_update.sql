UPDATE sso_audit
    SET "updated_at" = $2, "subject" = COALESCE("subject", $3), "data" = "data" || $4
    WHERE "id" = $1 AND CASE
        WHEN $5 IS NULL THEN TRUE
        ELSE "service_id" = $5
    END
    RETURNING "created_at", "updated_at", "id", "user_agent", "remote", "forwarded",
        "type" AS "type_", "subject", "data", "key_id", "service_id", "user_id", "user_key_id";
