-- Planned migration for a future SQLite-backed build.
-- Apply through the migration runner, not by executing the entire schema blindly.
-- It rebuilds tags so a general tag and creator tag may share the same display name.
PRAGMA foreign_keys = OFF;
BEGIN TRANSACTION;

CREATE TABLE tags_new (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    name     TEXT NOT NULL COLLATE NOCASE,
    tag_type TEXT NOT NULL DEFAULT 'general' CHECK (tag_type IN ('general', 'creator')),
    UNIQUE(name, tag_type)
);

INSERT INTO tags_new (id, name, tag_type)
SELECT id, name, 'general' FROM tags;

DROP TABLE tags;
ALTER TABLE tags_new RENAME TO tags;
CREATE INDEX IF NOT EXISTS idx_tags_type ON tags(tag_type);
CREATE INDEX IF NOT EXISTS idx_media_file_tags_tag_id ON media_file_tags(tag_id);

COMMIT;
PRAGMA foreign_keys = ON;
