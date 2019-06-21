CREATE TABLE auth_service (
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    service_id VARCHAR NOT NULL,
    service_is_active BOOLEAN NOT NULL,
    service_name VARCHAR NOT NULL,
    service_url VARCHAR NOT NULL,
    PRIMARY KEY (service_id)
);

CREATE TABLE auth_user (
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    user_id VARCHAR NOT NULL,
    user_is_active BOOLEAN NOT NULL,
    user_name VARCHAR NOT NULL,
    user_email VARCHAR NOT NULL,
    user_password_hash VARCHAR,
    PRIMARY KEY (user_id),
    CONSTRAINT uq_auth_user_email UNIQUE(user_email)
);

CREATE TABLE auth_key (
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    key_id VARCHAR NOT NULL,
    key_is_active BOOLEAN NOT NULL,
    key_name VARCHAR NOT NULL,
    key_value VARCHAR NOT NULL,
    service_id VARCHAR,
    user_id VARCHAR,
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
    created_at TIMESTAMPTZ NOT NULL,
    csrf_key VARCHAR NOT NULL,
    csrf_value VARCHAR NOT NULL,
    service_id VARCHAR NOT NULL,
    PRIMARY KEY (csrf_key),
    CONSTRAINT fk_auth_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE CASCADE
);

CREATE TABLE auth_audit (
    created_at TIMESTAMPTZ NOT NULL,
    audit_id VARCHAR NOT NULL,
    audit_user_agent VARCHAR NOT NULL,
    audit_remote VARCHAR NOT NULL,
    audit_forwarded_for VARCHAR,
    audit_key VARCHAR NOT NULL,
    audit_data JSONB NOT NULL,
    key_id VARCHAR NOT NULL,
    service_id VARCHAR,
    user_id VARCHAR,
    user_key_id VARCHAR,
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
