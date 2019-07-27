CREATE TABLE kv_disk (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    disk_id TEXT NOT NULL,
    disk_name TEXT NOT NULL,
    disk_options TEXT NOT NULL,
    PRIMARY KEY (disk_id),
    CONSTRAINT uq_kv_disk_name UNIQUE(disk_name)
);

CREATE TABLE kv_key (
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    key_id TEXT NOT NULL,
    key_name TEXT NOT NULL,
    disk_id TEXT NOT NULL,
    version_id TEXT,
    PRIMARY KEY (key_id),
    CONSTRAINT uq_kv_key_name UNIQUE(key_name, disk_id),
    CONSTRAINT fk_kv_key_disk
        FOREIGN KEY (disk_id)
        REFERENCES kv_disk(disk_id)
        ON DELETE RESTRICT,
    CONSTRAINT fk_kv_key_version
        FOREIGN KEY (version_id)
        REFERENCES kv_version(version_id)
        ON DELETE SET NULL
);

CREATE TABLE kv_version (
    created_at TEXT NOT NULL,
    version_id TEXT NOT NULL,
    version_hash BLOB NOT NULL,
    version_size INTEGER NOT NULL,
    key_id TEXT NOT NULL,
    PRIMARY KEY (version_id),
    CONSTRAINT fk_kv_version_key
        FOREIGN KEY (key_id)
        REFERENCES kv_key(key_id)
        ON DELETE RESTRICT
);

CREATE TABLE kv_data (
   data_chunk INTEGER NOT NULL,
   data_value BLOB NOT NULL,
   version_id TEXT NOT NULL,
   PRIMARY KEY (data_chunk, version_id),
   CONSTRAINT fk_kv_data_version
      FOREIGN KEY (version_id)
      REFERENCES kv_version(version_id)
      ON DELETE RESTRICT
);
