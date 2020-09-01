BEGIN;

DROP SCHEMA IF EXISTS "sso" CASCADE;

DROP TYPE IF EXISTS sso_code_target;
DROP TYPE IF EXISTS sso_oauth2_provider;
DROP TYPE IF EXISTS sso_oauth2_target;

COMMIT;
