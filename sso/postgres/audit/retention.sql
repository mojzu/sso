DELETE FROM
    sso.audit_table
WHERE
    "created_at" < (CURRENT_TIMESTAMP - ($1::INT * '1d'::INTERVAL))
