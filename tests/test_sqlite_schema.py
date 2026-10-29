"""Optional stdlib smoke tests for the runtime SQLite schema and key constraints.

Run from the project root with: python -m unittest tests/test_sqlite_schema.py
These tests validate the SQL contract; they do not replace cargo check or a Tauri integration test.
"""
from pathlib import Path
import sqlite3
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
MIGRATION = ROOT / "database" / "migrations" / "0003_runtime_sqlite_index.sql"

UPSERT_SQL = """INSERT INTO media_files (
    id, kind, original_name, relative_path, source_path, extension, mime_type,
    size_bytes, created_at_source, modified_at_source, imported_at,
    content_category, sha256, metadata_json, status
) VALUES (?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, NULL, '{}', 'available')
ON CONFLICT(relative_path) DO UPDATE SET
    kind = excluded.kind,
    original_name = excluded.original_name,
    extension = excluded.extension,
    mime_type = excluded.mime_type,
    size_bytes = excluded.size_bytes,
    created_at_source = excluded.created_at_source,
    modified_at_source = excluded.modified_at_source,
    content_category = excluded.content_category,
    status = CASE WHEN media_files.status = 'needs_review' THEN 'needs_review' ELSE 'available' END"""


class RuntimeSqliteSchemaTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.db_path = Path(self.temp_dir.name) / "test.sqlite3"
        self.db = sqlite3.connect(str(self.db_path))
        self.db.execute("PRAGMA foreign_keys = ON")
        self.db.executescript(MIGRATION.read_text(encoding="utf-8"))
        self.db.execute(
            "INSERT OR IGNORE INTO index_state (id, library_root, is_scanning, last_seen_file_count, last_missing_count) VALUES (1, '', 0, 0, 0)"
        )

    def tearDown(self):
        self.db.close()
        self.temp_dir.cleanup()

    def test_migration_creates_runtime_tables(self):
        tables = {row[0] for row in self.db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        self.assertTrue({"media_files", "tags", "media_file_tags", "app_settings", "index_state", "schema_migrations"}.issubset(tables))

    def test_upsert_preserves_enriched_metadata_and_review_state(self):
        row = ("IRL/Фото/a.jpg", "photo", "a.jpg", "IRL/Фото/a.jpg", ".jpg", "image/jpeg", 123, 1000, 2000, 3000, "irl")
        self.db.execute(UPSERT_SQL, row)
        self.db.execute("UPDATE media_files SET metadata_json = '{\"width\":1920}', status = 'needs_review' WHERE id = ?", (row[0],))
        self.db.execute(UPSERT_SQL, row)
        count, metadata_json, status = self.db.execute("SELECT COUNT(*), metadata_json, status FROM media_files").fetchone()
        self.assertEqual(count, 1)
        self.assertEqual(metadata_json, '{"width":1920}')
        self.assertEqual(status, "needs_review")

    def test_general_tag_editor_does_not_remove_creator_tags(self):
        relative = "IRL/Фото/a.jpg"
        self.db.execute(UPSERT_SQL, (relative, "photo", "a.jpg", relative, ".jpg", "image/jpeg", 12, 100, 200, 300, "irl"))
        self.db.execute("INSERT INTO tags(name, tag_type) VALUES ('pet', 'general')")
        self.db.execute("INSERT INTO tags(name, tag_type) VALUES ('Creator A', 'creator')")
        file_tags = self.db.execute("SELECT id FROM tags").fetchall()
        self.db.executemany("INSERT INTO media_file_tags(media_file_id, tag_id) VALUES (?, ?)", [(relative, tag_id) for (tag_id,) in file_tags])
        self.db.execute("DELETE FROM media_file_tags WHERE media_file_id = ? AND tag_id IN (SELECT id FROM tags WHERE tag_type = 'general')", (relative,))
        remaining = self.db.execute("SELECT t.name, t.tag_type FROM media_file_tags m JOIN tags t ON t.id = m.tag_id WHERE m.media_file_id = ?", (relative,)).fetchall()
        self.assertEqual(remaining, [("Creator A", "creator")])

    def test_missing_files_are_retained_and_foreign_keys_are_valid(self):
        relative = "IRL/Видео/missing.mp4"
        self.db.execute(UPSERT_SQL, (relative, "video", "missing.mp4", relative, ".mp4", "video/mp4", 24, 100, 200, 300, "irl"))
        self.db.execute("UPDATE media_files SET status='missing' WHERE relative_path=?", (relative,))
        self.assertEqual(self.db.execute("SELECT status FROM media_files WHERE id=?", (relative,)).fetchone()[0], "missing")
        self.assertEqual(self.db.execute("PRAGMA foreign_key_check").fetchall(), [])


if __name__ == "__main__":
    unittest.main()
