-- Authentication users table.
CREATE TABLE auth_user (
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id BIGSERIAL PRIMARY KEY NOT NULL,
    user_name VARCHAR NOT NULL,
    user_email VARCHAR NOT NULL,
    user_password VARCHAR,
    user_password_revision BIGINT,
    CONSTRAINT uq_auth_user_email UNIQUE(user_email)
);

-- Authentication services table.
CREATE TABLE auth_service (
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    service_id BIGSERIAL PRIMARY KEY NOT NULL,
    service_name VARCHAR NOT NULL,
    service_url VARCHAR NOT NULL
);

-- Authentication keys table.
CREATE TABLE auth_key (
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    key_id BIGSERIAL PRIMARY KEY NOT NULL,
    key_name VARCHAR NOT NULL,
    key_value VARCHAR NOT NULL,
    service_id BIGINT NOT NULL,
    user_id BIGINT,
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

-- Authentication csrf table.
CREATE TABLE auth_csrf (
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    csrf_key VARCHAR PRIMARY KEY NOT NULL,
    csrf_value VARCHAR NOT NULL,
    service_id BIGINT NOT NULL,
    CONSTRAINT fk_auth_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE CASCADE
);
