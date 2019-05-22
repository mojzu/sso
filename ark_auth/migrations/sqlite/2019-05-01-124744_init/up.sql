CREATE TABLE auth_user (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    user_id INTEGER PRIMARY KEY NOT NULL,
    user_name TEXT NOT NULL,
    user_email TEXT NOT NULL,
    user_password_hash TEXT,
    user_password_revision INTEGER,
    CONSTRAINT uq_auth_user_email UNIQUE(user_email)
);

CREATE TABLE auth_service (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    service_id INTEGER PRIMARY KEY NOT NULL,
    service_name TEXT NOT NULL,
    service_url TEXT NOT NULL
);

CREATE TABLE auth_key (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    key_id INTEGER PRIMARY KEY NOT NULL,
    key_name TEXT NOT NULL,
    key_value TEXT NOT NULL,
    service_id INTEGER,
    user_id INTEGER,
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
    csrf_key TEXT PRIMARY KEY NOT NULL,
    csrf_value TEXT NOT NULL,
    service_id INTEGER NOT NULL,
    CONSTRAINT fk_auth_csrf_service
        FOREIGN KEY (service_id)
        REFERENCES auth_service(service_id)
        ON DELETE CASCADE
);
