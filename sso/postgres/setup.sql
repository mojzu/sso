BEGIN;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE SCHEMA IF NOT EXISTS "sso";

CREATE OR REPLACE FUNCTION sso._trigger_updated_at_set(_tbl regclass)
RETURNS VOID
AS $$
BEGIN
    EXECUTE format('DROP TRIGGER IF EXISTS updated_at_trigger ON %s', _tbl);
    EXECUTE format('CREATE TRIGGER updated_at_trigger BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE sso._trigger_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sso._trigger_updated_at()
RETURNS trigger
AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW."updated_at" IS NOT DISTINCT FROM OLD."updated_at"
    ) THEN
        NEW."updated_at" := CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sso._secret_generate(
    p_count INT
)
RETURNS VARCHAR
LANGUAGE sql
STRICT
AS $$
    SELECT encode(gen_random_bytes(p_count), 'base64')
$$;

CREATE OR REPLACE FUNCTION sso._secret_hash(
    p_secret VARCHAR,
    p_value VARCHAR
)
RETURNS VARCHAR
LANGUAGE sql
STRICT
AS $$
    SELECT encode(hmac(p_value, p_secret, 'sha256'), 'base64')
$$;

CREATE OR REPLACE FUNCTION sso._secret_encrypt(
    p_encrypt_secret VARCHAR,
    p_hash_secret VARCHAR,
    p_value VARCHAR
)
RETURNS VARCHAR
LANGUAGE sql
STRICT
AS $$
    SELECT
        encode(
            encrypt(
                decode(
                    p_value || '.' || sso._secret_hash(p_hash_secret, p_value),
                    'escape'
                ),
                decode(p_encrypt_secret, 'escape'),
                'aes'
            ),
            'base64'
        )
$$;

CREATE OR REPLACE FUNCTION sso._secret_decrypt(
    p_encrypt_secret VARCHAR,
    p_value VARCHAR
)
RETURNS TABLE (
    value VARCHAR,
    hash VARCHAR
)
LANGUAGE sql
STRICT
AS $$
    SELECT
        split_part(key, '.', 1) as value,
        split_part(key, '.', 2) as hash
	FROM
        encode(
            decrypt(
                decode(p_value, 'base64'),
                decode(p_encrypt_secret, 'escape'),
                'aes'
            ),
            'escape'
        ) AS key
$$;

CREATE OR REPLACE FUNCTION sso._secret_check(
    p_hash_secret VARCHAR,
    p_hash VARCHAR,
    p_value VARCHAR
)
RETURNS BOOLEAN
LANGUAGE sql
STRICT
AS $$
    SELECT p_hash = sso._secret_hash(p_hash_secret, p_value)
$$;

CREATE OR REPLACE FUNCTION sso._password_hash(
    p_password VARCHAR
)
RETURNS VARCHAR
LANGUAGE sql
STRICT
AS $$
    SELECT crypt(p_password, gen_salt('bf'))
$$;

CREATE OR REPLACE FUNCTION sso._password_check(
    p_password VARCHAR,
    p_hash VARCHAR
)
RETURNS BOOLEAN
LANGUAGE sql
STRICT
AS $$
    SELECT p_hash = crypt(p_password, p_hash)
$$;

CREATE OR REPLACE FUNCTION sso._ttl_timestamptz(
    p_ttl_s BIGINT
)
RETURNS TIMESTAMPTZ
LANGUAGE sql
STRICT
AS $$
    SELECT (CURRENT_TIMESTAMP + (p_ttl_s * '1s'::INTERVAL))
$$;

DO $$ BEGIN
    CREATE TYPE sso_code_target AS ENUM ();
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
ALTER TYPE sso_code_target ADD VALUE IF NOT EXISTS 'Auth';
ALTER TYPE sso_code_target ADD VALUE IF NOT EXISTS 'PasswordReset';
ALTER TYPE sso_code_target ADD VALUE IF NOT EXISTS 'Register';
ALTER TYPE sso_code_target ADD VALUE IF NOT EXISTS 'Delete';

DO $$ BEGIN
    CREATE TYPE sso_oauth2_provider AS ENUM ();
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
ALTER TYPE sso_oauth2_provider ADD VALUE IF NOT EXISTS 'Sso';
ALTER TYPE sso_oauth2_provider ADD VALUE IF NOT EXISTS 'Microsoft';

DO $$ BEGIN
    CREATE TYPE sso_oauth2_target AS ENUM ();
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;
ALTER TYPE sso_oauth2_target ADD VALUE IF NOT EXISTS 'Auth';
ALTER TYPE sso_oauth2_target ADD VALUE IF NOT EXISTS 'Register';

CREATE TABLE IF NOT EXISTS sso.user_table (
    "created_at"  TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "updated_at"  TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "id"          UUID         NOT NULL  DEFAULT uuid_generate_v4()  PRIMARY KEY,
    "name"        VARCHAR      NOT NULL,
    "email"       VARCHAR      NOT NULL,
    "locale"      VARCHAR      NOT NULL  DEFAULT '',
    "timezone"    VARCHAR      NOT NULL  DEFAULT '',
    "enable"      BOOLEAN      NOT NULL  DEFAULT TRUE,
    "static"      BOOLEAN      NOT NULL  DEFAULT FALSE,
    CONSTRAINT user_name_check_length
        CHECK (LENGTH("name") > 0 AND LENGTH("name") <= 500),
    CONSTRAINT user_email_check_length
        CHECK (LENGTH("email") > 0 AND LENGTH("email") <= 1000),
    CONSTRAINT user_email_unique
        UNIQUE ("email"),
    CONSTRAINT user_locale_check_length
        CHECK (LENGTH("locale") <= 100),
    CONSTRAINT user_timezone_check_length
        CHECK (LENGTH("timezone") <= 500)
);
SELECT sso._trigger_updated_at_set('sso.user_table');

CREATE TABLE IF NOT EXISTS sso.user_password_table (
    "created_at"      TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "updated_at"      TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "user_id"         UUID         NOT NULL  PRIMARY KEY,
    "hash"            VARCHAR      NOT NULL,
    "allow_reset"     BOOLEAN      NOT NULL  DEFAULT FALSE,
    "require_update"  BOOLEAN      NOT NULL  DEFAULT FALSE,
    "static"          BOOLEAN      NOT NULL  DEFAULT FALSE,
    CONSTRAINT user_password_user_fk
        FOREIGN KEY ("user_id")
        REFERENCES sso.user_table("id")
        ON DELETE CASCADE,
    CONSTRAINT user_password_hash_check_length
        CHECK (LENGTH("hash") > 0 AND LENGTH("hash") <= 500)
);
SELECT sso._trigger_updated_at_set('sso.user_password_table');

CREATE TABLE IF NOT EXISTS sso.user_oauth2_provider_table (
    "created_at"       TIMESTAMPTZ          NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "user_id"          UUID                 NOT NULL,
    "oauth2_provider"  sso_oauth2_provider  NOT NULL,
    "sub"              VARCHAR              NOT NULL,
    "static"           BOOLEAN              NOT NULL  DEFAULT FALSE,
    PRIMARY KEY ("user_id", "oauth2_provider", "sub"),
    CONSTRAINT user_oauth2_provider_user_fk
        FOREIGN KEY ("user_id")
        REFERENCES sso.user_table("id")
        ON DELETE CASCADE,
    CONSTRAINT user_oauth2_provider_oauth2_provider_sub_unique
        UNIQUE ("oauth2_provider", "sub"),
    CONSTRAINT user_oauth2_provider_sub_check_length
        CHECK (LENGTH("sub") > 0 AND LENGTH("sub") <= 500)
);

CREATE TABLE IF NOT EXISTS sso.access_table (
    "created_at"  TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "updated_at"  TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "client_id"   UUID         NOT NULL,
    "user_id"     UUID         NOT NULL,
    "enable"      BOOLEAN      NOT NULL  DEFAULT TRUE,
    "scope"       VARCHAR      NOT NULL  DEFAULT '',
    "static"      BOOLEAN      NOT NULL  DEFAULT FALSE,
    PRIMARY KEY ("client_id", "user_id"),
    CONSTRAINT access_user_fk
        FOREIGN KEY ("user_id")
        REFERENCES sso.user_table("id")
        ON DELETE CASCADE,
    CONSTRAINT access_scope_check_length
        CHECK (LENGTH("scope") <= 1000)
);
SELECT sso._trigger_updated_at_set('sso.access_table');

CREATE TABLE IF NOT EXISTS sso.csrf_table (
    "created_at"  TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "client_id"   UUID         NOT NULL,
    "token"       VARCHAR      NOT NULL,
    "ttl"         TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP + '1h'::INTERVAL,
    PRIMARY KEY ("client_id", "token"),
    CONSTRAINT csrf_token_check_length
        CHECK (LENGTH("token") > 0 AND LENGTH ("token") <= 500)
);

CREATE TABLE IF NOT EXISTS sso.code_table (
    "created_at"  TIMESTAMPTZ      NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "client_id"   UUID             NOT NULL,
    "user_id"     UUID             NULL      DEFAULT NULL,
    "value"       VARCHAR          NOT NULL,
    "target"      sso_code_target  NOT NULL,
    "ttl"         TIMESTAMPTZ      NOT NULL  DEFAULT CURRENT_TIMESTAMP + '1h'::INTERVAL,
    "state"       VARCHAR          NOT NULL  DEFAULT '',
    "scope"       VARCHAR          NOT NULL  DEFAULT '',
    "email"       VARCHAR          NOT NULL  DEFAULT '',
    PRIMARY KEY ("client_id", "value"),
    CONSTRAINT code_user_fk
        FOREIGN KEY ("user_id")
        REFERENCES sso.user_table("id")
        ON DELETE CASCADE,
    CONSTRAINT code_access_fk
        FOREIGN KEY ("client_id", "user_id")
        REFERENCES sso.access_table("client_id", "user_id")
        ON DELETE CASCADE,
    CONSTRAINT code_value_check_length
        CHECK (LENGTH("value") > 0 AND LENGTH ("value") <= 500),
    CONSTRAINT code_state_check_length
        CHECK (LENGTH("state") <= 1000),
    CONSTRAINT code_scope_check_length
        CHECK (LENGTH("scope") <= 1000),
    CONSTRAINT code_email_check_length
        CHECK (LENGTH("email") <= 1000)
);

CREATE TABLE IF NOT EXISTS sso.oauth2_code_table (
    "created_at"    TIMESTAMPTZ          NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "client_id"     UUID                 NOT NULL,
    "csrf"          VARCHAR              NOT NULL,
    "provider"      sso_oauth2_provider  NOT NULL,
    "target"        sso_oauth2_target    NOT NULL,
    "ttl"           TIMESTAMPTZ          NOT NULL  DEFAULT CURRENT_TIMESTAMP + '1h'::INTERVAL,
    "pkce"          VARCHAR              NOT NULL  DEFAULT '',
    "redirect_uri"  VARCHAR              NOT NULL  DEFAULT '',
    "state"         VARCHAR              NOT NULL  DEFAULT '',
    "scope"         VARCHAR              NOT NULL  DEFAULT '',
    "email"         VARCHAR              NOT NULL  DEFAULT '',
    PRIMARY KEY ("client_id", "csrf"),
    CONSTRAINT oauth2_code_csrf_check_length
        CHECK (LENGTH("csrf") > 0 AND LENGTH ("csrf") <= 1000),
    CONSTRAINT oauth2_code_pkce_check_length
        CHECK (LENGTH ("pkce") <= 1000),
    CONSTRAINT oauth2_code_redirect_uri_check_length
        CHECK (LENGTH ("redirect_uri") <= 1000),
    CONSTRAINT oauth2_code_state_check_length
        CHECK (LENGTH ("state") <= 1000),
    CONSTRAINT oauth2_code_scope_check_length
        CHECK (LENGTH ("scope") <= 1000),
    CONSTRAINT oauth2_code_email_check_length
        CHECK (LENGTH("email") <= 1000)
);

CREATE TABLE IF NOT EXISTS sso.token_table (
    "created_at"     TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "updated_at"     TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "id"             UUID         NOT NULL  DEFAULT uuid_generate_v4()  PRIMARY KEY,
    "client_id"      UUID         NOT NULL,
    "user_id"        UUID         NOT NULL,
    "ttl"            TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP + '1h'::INTERVAL,
    "value"          VARCHAR      NOT NULL,
    "value_refresh"  VARCHAR      NOT NULL,
    "name"           VARCHAR      NOT NULL  DEFAULT '',
    "enable"         BOOLEAN      NOT NULL  DEFAULT TRUE,
    "scope"          VARCHAR      NOT NULL  DEFAULT '',
    CONSTRAINT token_user_fk
        FOREIGN KEY ("user_id")
        REFERENCES sso.user_table("id")
        ON DELETE CASCADE,
    CONSTRAINT token_access_fk
        FOREIGN KEY ("client_id", "user_id")
        REFERENCES sso.access_table("client_id", "user_id")
        ON DELETE CASCADE,
    CONSTRAINT token_value_check_length
        CHECK (LENGTH("value") > 0 AND LENGTH ("value") <= 500),
    CONSTRAINT token_value_refresh_check_length
        CHECK (LENGTH("value_refresh") > 0 AND LENGTH ("value_refresh") <= 500),
    CONSTRAINT token_name_check_length
        CHECK (LENGTH("name") <= 500),
    CONSTRAINT token_scope_check_length
        CHECK (LENGTH("scope") <= 1000)
);
SELECT sso._trigger_updated_at_set('sso.token_table');

CREATE TABLE IF NOT EXISTS sso.api_key_table (
    "created_at"     TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "updated_at"     TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "id"             UUID         NOT NULL  DEFAULT uuid_generate_v4()  PRIMARY KEY,
    "client_id"      UUID         NOT NULL,
    "user_id"        UUID         NOT NULL,
    "value"          VARCHAR      NOT NULL,
    "name"           VARCHAR      NOT NULL  DEFAULT '',
    "enable"         BOOLEAN      NOT NULL  DEFAULT TRUE,
    "scope"          VARCHAR      NOT NULL  DEFAULT '',
    CONSTRAINT api_key_user_fk
        FOREIGN KEY ("user_id")
        REFERENCES sso.user_table("id")
        ON DELETE CASCADE,
    CONSTRAINT api_key_access_fk
        FOREIGN KEY ("client_id", "user_id")
        REFERENCES sso.access_table("client_id", "user_id")
        ON DELETE CASCADE,
    CONSTRAINT api_key_value_check_length
        CHECK (LENGTH("value") > 0 AND LENGTH ("value") <= 500),
    CONSTRAINT api_key_name_check_length
        CHECK (LENGTH("name") <= 500),
    CONSTRAINT api_key_scope_check_length
        CHECK (LENGTH("scope") <= 1000)
);
SELECT sso._trigger_updated_at_set('sso.api_key_table');

CREATE TABLE IF NOT EXISTS sso.audit_table (
    "created_at"   TIMESTAMPTZ  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
    "id"           BIGSERIAL                                         PRIMARY KEY,
    "client_id"    UUID         NULL      DEFAULT NULL,
    "user_id"      UUID         NULL      DEFAULT NULL,
    "token_id"     UUID         NULL      DEFAULT NULL,
    "api_key_id"   UUID         NULL      DEFAULT NULL,
    "type"         VARCHAR      NOT NULL,
    "subject"      VARCHAR      NULL      DEFAULT NULL,
    "data"         JSONB        NOT NULL  DEFAULT '{}'::JSONB,
    "status_code"  SMALLINT     NULL      DEFAULT NULL,
    CONSTRAINT audit_type_check_length
        CHECK (LENGTH("type") > 0 AND LENGTH("type") <= 100),
    CONSTRAINT audit_subject_check_length
        CHECK (LENGTH("subject") <= 1000)
);
CREATE INDEX IF NOT EXISTS audit_created_at_type_index ON sso.audit_table("created_at" DESC, "type");

COMMIT;
