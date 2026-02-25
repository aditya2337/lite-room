# `import` End-to-End Flow

This is the highest-value path to understand first.

## 1. Driver receives command
File:
- [/lite-room/crates/drivers/src/main.rs](../../crates/drivers/src/main.rs)

Flow:
1. `parse_command()` returns `Command::Import { folder }`.
2. `run_command()` calls:
   - `service.import_folder(ImportFolderCommand { folder, cache_root })`

## 2. Application orchestrates use-case
File:
- [/lite-room/crates/application/src/service.rs](../../crates/application/src/service.rs)

Inside `import_folder()`:
1. Validate non-empty `folder` and `cache_root`.
2. Call `scanner.scan_supported(folder)` via `FileScanner`.
3. Build default `EditParams`, validate, serialize to JSON.
4. Iterate supported files:
   - build `metadata_json`
   - `catalog.upsert_image(...)`
   - `catalog.ensure_default_edit(...)`
   - `thumbnails.ensure_thumbnail(...)`
   - `catalog.upsert_thumbnail(...)`
5. Return `ImportReport`.

## 3. Filesystem scanner adapter
File:
- [/lite-room/crates/adapters/src/fs/scanner.rs](../../crates/adapters/src/fs/scanner.rs)

Responsibilities:
1. Validate folder path is a directory.
2. Walk files recursively using `walkdir`.
3. Filter unsupported kinds via domain `detect_image_kind`.
4. Produce `FileScanSummary`.

## 4. SQLite catalog adapter
Files:
- [/lite-room/crates/adapters/src/sqlite/mod.rs](../../crates/adapters/src/sqlite/mod.rs)
- [/lite-room/crates/adapters/src/sqlite/queries.rs](../../crates/adapters/src/sqlite/queries.rs)

Responsibilities:
1. Open DB connection.
2. Upsert `images` row by `file_path`.
3. Ensure default `edits` row exists.
4. Upsert `thumbnails` row.

## 5. Thumbnail adapter
File:
- [/lite-room/crates/adapters/src/fs/thumbs.rs](../../crates/adapters/src/fs/thumbs.rs)

Responsibilities:
1. Build cache path `<cache_root>/thumbs/<image_id>.jpg`.
2. JPEG: decode and generate thumbnail.
3. RAW/unsupported: generate placeholder thumbnail.
4. Return `ThumbnailArtifact`.

## 6. Schema and migrations
Files:
- [/lite-room/crates/adapters/src/migrations/mod.rs](../../crates/adapters/src/migrations/mod.rs)
- [/lite-room/crates/adapters/src/migrations/0001_initial.sql](../../crates/adapters/src/migrations/0001_initial.sql)
