-- Черновая схема SQLite для следующего этапа разработки.
-- Этот файл пока НЕ исполняется приложением: экран базы использует demoMedia.ts.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS media_files (
    id                  TEXT PRIMARY KEY,
    kind                TEXT NOT NULL CHECK (kind IN ('video', 'photo', 'archive')),
    original_name       TEXT NOT NULL,
    relative_path       TEXT NOT NULL UNIQUE,
    source_path         TEXT,
    extension           TEXT,
    mime_type           TEXT,
    size_bytes          INTEGER CHECK (size_bytes IS NULL OR size_bytes >= 0),
    created_at_source   TEXT,
    modified_at_source  TEXT,
    imported_at         TEXT NOT NULL,
    content_category    TEXT CHECK (content_category IN ('3d', 'anime', 'irl')),
    sha256              TEXT,
    metadata_json       TEXT NOT NULL DEFAULT '{}',
    status              TEXT NOT NULL DEFAULT 'available'
                            CHECK (status IN ('available', 'missing', 'needs_review'))
);

CREATE INDEX IF NOT EXISTS idx_media_files_kind ON media_files(kind);
CREATE INDEX IF NOT EXISTS idx_media_files_category ON media_files(content_category);
CREATE INDEX IF NOT EXISTS idx_media_files_imported_at ON media_files(imported_at);

CREATE TABLE IF NOT EXISTS tags (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT NOT NULL COLLATE NOCASE,
    tag_type TEXT NOT NULL DEFAULT 'general' CHECK (tag_type IN ('general', 'creator')),
    UNIQUE(name, tag_type)
);

CREATE TABLE IF NOT EXISTS media_file_tags (
    media_file_id   TEXT NOT NULL REFERENCES media_files(id) ON DELETE CASCADE,
    tag_id          INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (media_file_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_media_file_tags_tag_id ON media_file_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_tags_type ON tags(tag_type);

CREATE TABLE IF NOT EXISTS app_settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);

-- Важно: relative_path хранится относительно корня выбранной медиатеки.
-- Это упростит перенос медиатеки на другой диск без переписывания всех записей.

-- Creator/author records are represented by tags.tag_type='creator'.
-- A future Authors page can list these tags and the associated media_file_tags rows.
