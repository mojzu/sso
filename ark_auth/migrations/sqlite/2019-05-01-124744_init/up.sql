CREATE TABLE auth_user (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    user_active BOOLEAN NOT NULL,
    user_password_hash TEXT,
    user_password_revision INTEGER,
    PRIMARY KEY (user_id)
    CONSTRAINT uq_auth_user_email UNIQUE(user_email)
);

CREATE TABLE auth_service (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    service_id INTEGER NOT NULL,
    service_name TEXT NOT NULL,
    service_url TEXT NOT NULL,
    PRIMARY KEY (service_id)
);

CREATE TABLE auth_key (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    key_id INTEGER NOT NULL,
    key_name TEXT NOT NULL,
    key_value TEXT NOT NULL,
    service_id INTEGER,
    user_id INTEGER,
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
    service_id INTEGER NOT NULL,
    PRIMARY KEY (csrf_key),
    CONSTRAINT fk_auth_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE CASCADE
);
