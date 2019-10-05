CREATE TABLE sso_service (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    service_id TEXT NOT NULL,
    service_is_enabled BOOLEAN NOT NULL,
    service_name TEXT NOT NULL,
    service_url TEXT NOT NULL,
    PRIMARY KEY (service_id)
);

CREATE TABLE sso_user (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    user_id TEXT NOT NULL,
    user_is_enabled BOOLEAN NOT NULL,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    user_password_hash TEXT,
    PRIMARY KEY (user_id)
    CONSTRAINT uq_sso_user_email UNIQUE(user_email)
);

CREATE TABLE sso_key (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    key_id TEXT NOT NULL,
    key_is_enabled BOOLEAN NOT NULL,
    key_is_revoked BOOLEAN NOT NULL,
    key_name TEXT NOT NULL,
    key_value TEXT NOT NULL,
    service_id TEXT,
    user_id TEXT,
    PRIMARY KEY (key_id),
    CONSTRAINT uq_sso_key_value UNIQUE(key_value),
    CONSTRAINT fk_sso_key_service
        FOREIGN KEY (service_id)
        REFERENCES sso_service(service_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_key_user
        FOREIGN KEY (user_id)
        REFERENCES sso_user(user_id)
        ON DELETE RESTRICT
);

CREATE TABLE sso_csrf (
    created_at TEXT NOT NULL,
    csrf_key TEXT NOT NULL,
    csrf_value TEXT NOT NULL,
    csrf_ttl TEXT NOT NULL,
    service_id TEXT NOT NULL,
    PRIMARY KEY (csrf_key),
    CONSTRAINT fk_sso_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES sso_service(service_id)
        ON DELETE CASCADE
);

CREATE TABLE sso_audit (
    created_at TEXT NOT NULL,
    audit_id TEXT NOT NULL,
    audit_user_agent TEXT NOT NULL,
    audit_remote TEXT NOT NULL,
    audit_forwarded TEXT,
    audit_type TEXT NOT NULL,
    audit_data BLOB NOT NULL,
    key_id TEXT,
    service_id TEXT,
    user_id TEXT,
    user_key_id TEXT,
    PRIMARY KEY (audit_id),
    CONSTRAINT fk_sso_audit_key
        FOREIGN KEY (key_id)
        REFERENCES sso_key(key_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_audit_service
        FOREIGN KEY (service_id)
        REFERENCES sso_service(service_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_audit_user
        FOREIGN KEY (user_id)
        REFERENCES sso_user(user_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_audit_user_key
        FOREIGN KEY (user_key_id)
        REFERENCES sso_key(key_id)
        ON DELETE RESTRICT
);
CREATE INDEX idx_sso_audit_created_at ON sso_audit(created_at);
