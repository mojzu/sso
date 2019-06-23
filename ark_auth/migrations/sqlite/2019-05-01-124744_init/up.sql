CREATE TABLE auth_service (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    service_id TEXT NOT NULL,
    service_is_enabled BOOLEAN NOT NULL,
    service_name TEXT NOT NULL,
    service_url TEXT NOT NULL,
    PRIMARY KEY (service_id)
);

CREATE TABLE auth_user (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    user_id TEXT NOT NULL,
    user_is_enabled BOOLEAN NOT NULL,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    user_password_hash TEXT,
    PRIMARY KEY (user_id)
    CONSTRAINT uq_auth_user_email UNIQUE(user_email)
);

CREATE TABLE auth_key (
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
    CONSTRAINT uq_auth_key_value UNIQUE(key_value),
    CONSTRAINT fk_auth_key_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_auth_key_user
        FOREIGN KEY (user_id)
        REFERENCES auth_user(user_id)
        ON DELETE RESTRICT
);

CREATE TABLE auth_csrf (
    created_at TEXT NOT NULL,
    csrf_key TEXT NOT NULL,
    csrf_value TEXT NOT NULL,
    csrf_ttl TEXT NOT NULL,
    service_id TEXT NOT NULL,
    PRIMARY KEY (csrf_key),
    CONSTRAINT fk_auth_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE CASCADE
);

CREATE TABLE auth_audit (
    created_at TEXT NOT NULL,
    audit_id TEXT NOT NULL,
    audit_user_agent TEXT NOT NULL,
    audit_remote TEXT NOT NULL,
    audit_forwarded_for TEXT,
    audit_path TEXT NOT NULL,
    audit_data BLOB NOT NULL,
    key_id TEXT NOT NULL,
    service_id TEXT,
    user_id TEXT,
    user_key_id TEXT,
    PRIMARY KEY (audit_id),
    CONSTRAINT fk_auth_audit_key
        FOREIGN KEY (key_id)
        REFERENCES auth_key(key_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_auth_audit_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_auth_audit_user
        FOREIGN KEY (user_id)
        REFERENCES auth_user(user_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_auth_audit_user_key
        FOREIGN KEY (user_key_id)
        REFERENCES auth_key(key_id)
        ON DELETE RESTRICT
);
