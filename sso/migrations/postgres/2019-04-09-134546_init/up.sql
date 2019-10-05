CREATE TABLE sso_service (
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    service_id UUID NOT NULL,
    service_is_enabled BOOLEAN NOT NULL,
    service_name VARCHAR NOT NULL,
    service_url VARCHAR NOT NULL,
    service_provider_local_url VARCHAR,
    service_provider_github_oauth2_url VARCHAR,
    service_provider_microsoft_oauth2_url VARCHAR,
    PRIMARY KEY (service_id)
);

CREATE TABLE sso_user (
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    user_id UUID NOT NULL,
    user_is_enabled BOOLEAN NOT NULL,
    user_name VARCHAR NOT NULL,
    user_email VARCHAR NOT NULL,
    user_locale VARCHAR NOT NULL,
    user_timezone VARCHAR NOT NULL,
    user_password_allow_reset BOOLEAN NOT NULL,
    user_password_require_update BOOLEAN NOT NULL,
    user_password_hash VARCHAR,
    PRIMARY KEY (user_id),
    CONSTRAINT uq_sso_user_email UNIQUE(user_email)
);

CREATE TABLE sso_key (
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    key_id UUID NOT NULL,
    key_is_enabled BOOLEAN NOT NULL,
    key_is_revoked BOOLEAN NOT NULL,
    key_type VARCHAR NOT NULL,
    key_name VARCHAR NOT NULL,
    key_value VARCHAR NOT NULL,
    service_id UUID,
    user_id UUID,
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
CREATE UNIQUE INDEX idx_sso_key_type_token ON sso_key (service_id, user_id)
    WHERE key_is_enabled IS TRUE AND key_type = 'Token';
CREATE UNIQUE INDEX idx_sso_key_type_totp ON sso_key (service_id, user_id)
    WHERE key_is_enabled IS TRUE AND key_type = 'Totp';

CREATE TABLE sso_csrf (
    created_at TIMESTAMPTZ NOT NULL,
    csrf_key VARCHAR NOT NULL,
    csrf_value VARCHAR NOT NULL,
    csrf_ttl TIMESTAMPTZ NOT NULL,
    service_id UUID NOT NULL,
    PRIMARY KEY (csrf_key),
    CONSTRAINT fk_sso_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES sso_service(service_id)
        ON DELETE CASCADE
);

CREATE TABLE sso_audit (
    created_at TIMESTAMPTZ NOT NULL,
    audit_id UUID NOT NULL,
    audit_user_agent VARCHAR NOT NULL,
    audit_remote VARCHAR NOT NULL,
    audit_forwarded VARCHAR,
    audit_type VARCHAR NOT NULL,
    audit_data JSONB NOT NULL,
    key_id UUID,
    service_id UUID,
    user_id UUID,
    user_key_id UUID,
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
CREATE INDEX idx_sso_audit_created_at ON sso_audit(created_at DESC, audit_type);
