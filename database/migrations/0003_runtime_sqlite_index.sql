-- Migration 0003: runtime SQLite index.
-- This is a self-contained baseline because previous migrations were planning drafts and never ran in the shipped prototype.
-- Relative paths are always relative to the configured media-library root; absolute paths are resolved at query time.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS media_files (
    id                  TEXT PRIMARY KEY,
    kind                TEXT NOT NULL CHECK (kind IN ('video', 'photo', 'archive')),
    original_name       TEXT NOT NULL,
    relative_path       TEXT NOT NULL UNIQUE,
    source_path         TEXT,
    extension           TEXT,
    mime_type           TEXT,
    size_bytes          INTEGER NOT NULL CHECK (size_bytes >= 0),
    created_at_source   INTEGER,
    modified_at_source  INTEGER,
    imported_at         INTEGER NOT NULL,
    content_category    TEXT CHECK (content_category IN ('3d', 'anime', 'irl')),
    sha256              TEXT,
    metadata_json       TEXT NOT NULL DEFAULT '{}',
    status              TEXT NOT NULL DEFAULT 'available'
                            CHECK (status IN ('available', 'missing', 'needs_review'))
);

CREATE INDEX IF NOT EXISTS idx_media_files_kind ON media_files(kind);
CREATE INDEX IF NOT EXISTS idx_media_files_category ON media_files(content_category);
CREATE INDEX IF NOT EXISTS idx_media_files_imported_at ON media_files(imported_at);
CREATE INDEX IF NOT EXISTS idx_media_files_status ON media_files(status);
CREATE INDEX IF NOT EXISTS idx_media_files_modified ON media_files(modified_at_source DESC);

CREATE TABLE IF NOT EXISTS tags (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL COLLATE NOCASE,
    tag_type TEXT NOT NULL DEFAULT 'general' CHECK (tag_type IN ('general', 'creator')),
    UNIQUE(name, tag_type)
);

CREATE TABLE IF NOT EXISTS media_file_tags (
    media_file_id TEXT NOT NULL REFERENCES media_files(id) ON DELETE CASCADE,
    tag_id        INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (media_file_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_media_file_tags_tag_id ON media_file_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_tags_type ON tags(tag_type);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS index_state (
    id                    INTEGER PRIMARY KEY CHECK (id = 1),
    library_root          TEXT NOT NULL DEFAULT '',
    is_scanning           INTEGER NOT NULL DEFAULT 0 CHECK (is_scanning IN (0, 1)),
    last_started_at       INTEGER,
    last_completed_at     INTEGER,
    last_full_scan_at     INTEGER,
    last_seen_file_count  INTEGER NOT NULL DEFAULT 0,
    last_missing_count    INTEGER NOT NULL DEFAULT 0,
    last_error            TEXT
);

CREATE TABLE IF NOT EXISTS schema_migrations (
    version    INTEGER PRIMARY KEY,
    applied_at INTEGER NOT NULL
);

-- Creator/author records use tags.tag_type='creator'.
-- Technical metadata can be added to metadata_json and normalized columns as extractors are implemented.
