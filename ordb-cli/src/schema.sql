CREATE TABLE IF NOT EXISTS files (
    id            INTEGER PRIMARY KEY,
    source_path   TEXT NOT NULL UNIQUE,
    file_size     INTEGER NOT NULL,
    mime_type     TEXT,
    blake3_hash   TEXT,
    status        TEXT NOT NULL DEFAULT 'PENDIENTE', 
    primary_id    INTEGER REFERENCES files(id),
    category      TEXT,
    confidence    REAL,
    date_source   TEXT,
    date_value    TEXT,
    dest_path     TEXT,
    artist        TEXT,
    album         TEXT,
    error_msg     TEXT,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_hash ON files(blake3_hash);
CREATE INDEX IF NOT EXISTS idx_status ON files(status);
CREATE TABLE IF NOT EXISTS sources (
    id   INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE
);
