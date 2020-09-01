SELECT
    "created_at",
    "id",
    "client_id",
    "user_id",
    "token_id",
    "api_key_id",
    "type",
    "subject",
    "data",
    "status_code"
FROM
    sso.audit_table
WHERE
    "client_id" = $1
AND
    CASE WHEN ($2::BIGINT IS NULL) THEN
        TRUE
    ELSE
        "id" < $2
    END
AND
    CASE WHEN ($4::BIGINT[] IS NULL) THEN
        TRUE
    ELSE
        "id" = ANY($4)
    END
AND
    CASE WHEN ($5::UUID[] IS NULL) THEN
        TRUE
    ELSE
        "user_id" = ANY($5)
    END
AND
    CASE WHEN ($6::VARCHAR[] IS NULL) THEN
        TRUE
    ELSE
        "type" = ANY($6)
    END
AND
    CASE WHEN ($7::VARCHAR[] IS NULL) THEN
        TRUE
    ELSE
        "subject" = ANY($7)
    END
ORDER BY
    "id" DESC
LIMIT
    $3::BIGINT
