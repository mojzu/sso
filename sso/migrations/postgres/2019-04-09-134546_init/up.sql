CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

CREATE TABLE sso_service (
    "created_at"                    TIMESTAMPTZ NOT NULL,
    "updated_at"                    TIMESTAMPTZ NOT NULL,
    "id"                            UUID        NOT NULL,
    "is_enabled"                    BOOLEAN     NOT NULL,
    "name"                          VARCHAR     NOT NULL,
    "url"                           VARCHAR     NOT NULL,
    "provider_local_url"            VARCHAR,
    "provider_github_oauth2_url"    VARCHAR,
    "provider_microsoft_oauth2_url" VARCHAR,
    PRIMARY KEY ("id")
);

CREATE TABLE sso_user (
    "created_at"              TIMESTAMPTZ NOT NULL,
    "updated_at"              TIMESTAMPTZ NOT NULL,
    "id"                      UUID        NOT NULL,
    "is_enabled"              BOOLEAN     NOT NULL,
    "name"                    VARCHAR     NOT NULL,
    "email"                   VARCHAR     NOT NULL,
    "locale"                  VARCHAR     NOT NULL,
    "timezone"                VARCHAR     NOT NULL,
    "password_allow_reset"    BOOLEAN     NOT NULL,
    "password_require_update" BOOLEAN     NOT NULL,
    "password_hash"           VARCHAR,
    PRIMARY KEY ("id"),
    CONSTRAINT uq_sso_user_email UNIQUE("email")
);

CREATE TABLE sso_key (
    "created_at" TIMESTAMPTZ NOT NULL,
    "updated_at" TIMESTAMPTZ NOT NULL,
    "id"         UUID        NOT NULL,
    "is_enabled" BOOLEAN     NOT NULL,
    "is_revoked" BOOLEAN     NOT NULL,
    "type"       VARCHAR     NOT NULL,
    "name"       VARCHAR     NOT NULL,
    "value"      VARCHAR     NOT NULL,
    "service_id" UUID,
    "user_id"    UUID,
    PRIMARY KEY ("id"),
    CONSTRAINT uq_sso_key_value UNIQUE("value"),
    CONSTRAINT fk_sso_key_service
        FOREIGN KEY ("service_id")
        REFERENCES sso_service("id")
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_key_user
        FOREIGN KEY ("user_id")
        REFERENCES sso_user("id")
        ON DELETE RESTRICT
);
CREATE UNIQUE INDEX idx_sso_key_type_token ON sso_key ("service_id", "user_id")
    WHERE "is_enabled" IS TRUE AND "type" = 'Token';
CREATE UNIQUE INDEX idx_sso_key_type_totp ON sso_key ("service_id", "user_id")
    WHERE "is_enabled" IS TRUE AND "type" = 'Totp';

CREATE TABLE sso_csrf (
    "created_at" TIMESTAMPTZ NOT NULL,
    "key"        VARCHAR     NOT NULL,
    "value"      VARCHAR     NOT NULL,
    "ttl"        TIMESTAMPTZ NOT NULL,
    "service_id" UUID        NOT NULL,
    PRIMARY KEY ("key"),
    CONSTRAINT fk_sso_csrf_service
        FOREIGN KEY ("service_id")
        REFERENCES sso_service("id")
        ON DELETE CASCADE
);

CREATE TABLE sso_audit (
    "created_at"  TIMESTAMPTZ NOT NULL,
    "updated_at" TIMESTAMPTZ NOT NULL,
    "id"          UUID        NOT NULL,
    "user_agent"  VARCHAR     NOT NULL,
    "remote"      VARCHAR     NOT NULL,
    "forwarded"   VARCHAR,
    "type"        VARCHAR     NOT NULL,
    "subject"     VARCHAR,
    "data"        JSONB       NOT NULL,
    "key_id"      UUID,
    "service_id"  UUID,
    "user_id"     UUID,
    "user_key_id" UUID,
    PRIMARY KEY ("created_at", "id"),
    CONSTRAINT fk_sso_audit_key
        FOREIGN KEY ("key_id")
        REFERENCES sso_key("id")
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_audit_service
        FOREIGN KEY ("service_id")
        REFERENCES sso_service("id")
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_audit_user
        FOREIGN KEY ("user_id")
        REFERENCES sso_user("id")
        ON DELETE RESTRICT,
    CONSTRAINT fk_sso_audit_user_key
        FOREIGN KEY ("user_key_id")
        REFERENCES sso_key("id")
        ON DELETE RESTRICT
);
SELECT create_hypertable('sso_audit', 'created_at');
CREATE INDEX idx_sso_audit_created_at ON sso_audit("created_at" DESC, "type");
