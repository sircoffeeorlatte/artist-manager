# Runtime SQLite index

## DB file and migrations

Desktop mode stores `media-library.sqlite3` in Tauri's application data directory. The DB is intentionally not stored inside the media root, so choosing a different drive for media does not move the database or mix it with media files.

`src-tauri/src/commands/database.rs` opens the DB using bundled `rusqlite` SQLite. On startup it applies the self-contained migration `database/migrations/0003_runtime_sqlite_index.sql` if the migration version is below 3. The migration is recorded in `schema_migrations` and in SQLite's `user_version`. The migration is self-contained because the older numbered SQL files were drafts from before the runtime DB was enabled.

## Tables

- `media_files`: one row per indexed file, with stable-within-library ID/relative path, kind, name, extension, MIME type, file size, source timestamps, imported timestamp, category, optional JSON metadata, and availability status.
- `tags`: unique tag labels plus type `general` or `creator`.
- `media_file_tags`: many-to-many relation between files and tags.
- `index_state`: root path, scanning state, last scan start/completion, counts and latest error.
- `app_settings`: database-level settings and schema metadata reserved for future use.
- `schema_migrations`: applied migration versions.

All file timestamps are Unix epoch seconds; the frontend formatter converts them to local dates. Media paths stored in `relative_path` always use `/`. The native layer builds the absolute path from the selected library root only when returning an item to Vue.

## Scan behavior

`list_library_media` and `list_indexed_media` call `ensure_indexed`. It performs an initial scan when there is no completed index, scans when the selected root changes, and retries automatically after an interrupted previous process. Ordinary page-to-page navigation just runs a SQL query. If a scan returns a real filesystem/SQLite error, subsequent gallery requests show the saved error instead of launching another expensive scan on every page; retry explicitly with «Переиндексировать» on «База файлов».

The indexer scans the known paths under `IRL`, `Аниме`, and `3D`, with `Фото`, `Видео`, and `Архивы` children. Nested folders are included. It ignores symbolic links, unsupported extensions and temporary import files. Errors while reading a directory or reading file metadata abort the scan before the DB update transaction starts.

After the filesystem pass completes, a single transaction upserts discovered files and marks previously indexed but no-longer-discovered paths as `missing`. Missing rows are not deleted; tags and metadata remain available for recovery if the file returns. `metadata_json` is intentionally not overwritten by a basic scan, preserving metadata added by future extractors.

After an import succeeds, the import command triggers an index update. A manual full rescan is available from «База файлов». The database page displays the DB path, last scan time, available/missing counts and index errors.

## Tags

The database page currently edits general tags. Updating general tags does not delete links to `creator` tags, so future author metadata can share the relation table without being erased by the general tag editor. Tags are persisted in SQLite and loaded into gallery/database records.

## Current limits

- This version uses explicit full scans, not a live filesystem watcher.
- It supports one configured active media root at a time; it doesn't maintain parallel independent catalogs for multiple roots.
- File IDs are based on relative path. A later relocation workflow should distinguish moving the same library from switching to a different collection when both roots have identical relative paths.
- There is no checksum-based identity detection yet. The index detects changes via size and modified timestamp.
- UI and Rust builds must be verified on Windows with `check.cmd` and `cargo check --manifest-path src-tauri/Cargo.toml`.

## SQL smoke tests

The optional `tests/test_sqlite_schema.py` uses only Python's standard `sqlite3` module. Run `python -m unittest tests/test_sqlite_schema.py` from the project root to verify that the migration creates required tables, repeated upserts do not duplicate records, enriched JSON metadata and `needs_review` status are preserved, creator tags survive edits to general tags, and foreign-key links stay valid. This is a SQL-contract test, not a substitute for testing the Rust command layer on Windows.
