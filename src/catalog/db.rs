use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use image::{ImageFormat, ImageReader};
use rusqlite::{params, Connection};
use serde_json::json;
use walkdir::WalkDir;

use crate::cache::thumbs::thumbnail_path;
use crate::catalog::migrations::MIGRATIONS;
use crate::catalog::models::{ImageRecord, ImportReport};
use crate::catalog::queries;

#[derive(Debug, Clone)]
pub struct CatalogDb {
    path: PathBuf,
}

impl CatalogDb {
    pub fn new(path: String) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    pub fn initialize(&self) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            return Err("catalog path must not be empty".to_string());
        }

        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create catalog directory: {error}"))?;
            }
        }

        let conn = self.open_connection()?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")
            .map_err(|error| format!("failed to initialize pragmas: {error}"))?;

        for migration in MIGRATIONS {
            conn.execute_batch(migration)
                .map_err(|error| format!("failed to apply migration: {error}"))?;
        }

        Ok(())
    }

    pub fn import_jpegs_from_folder(
        &self,
        folder: &str,
        cache_root: &str,
    ) -> Result<ImportReport, String> {
        let folder_path = Path::new(folder);
        if !folder_path.is_dir() {
            return Err(format!(
                "folder does not exist or is not a directory: {folder}"
            ));
        }

        let thumbs_dir = Path::new(cache_root).join("thumbs");
        fs::create_dir_all(&thumbs_dir)
            .map_err(|error| format!("failed to create thumbnail cache directory: {error}"))?;

        let conn = self.open_connection()?;
        let now = now_iso_like();
        let default_edit = default_edit_params_json()?;

        let mut report = ImportReport::default();

        for entry in WalkDir::new(folder_path).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            report.scanned_files += 1;

            let file_path = entry.path();
            if !is_supported_jpeg(file_path) {
                continue;
            }

            report.supported_files += 1;
            let canonical = file_path
                .canonicalize()
                .map_err(|error| format!("failed to canonicalize path {:?}: {error}", file_path))?;
            let canonical_str = canonical.to_string_lossy().to_string();
            let metadata = file_path
                .metadata()
                .map_err(|error| format!("failed to read metadata for {:?}: {error}", file_path))?;

            let metadata_json = json!({
                "file_size": metadata.len(),
                "extension": file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or_default()
                    .to_ascii_lowercase(),
            })
            .to_string();

            let inserted = conn
                .execute(
                    "INSERT OR IGNORE INTO images
                     (file_path, import_date, capture_date, camera_model, iso, rating, flag, metadata_json)
                     VALUES (?1, ?2, NULL, NULL, NULL, 0, 0, ?3)",
                    params![canonical_str, now, metadata_json],
                )
                .map_err(|error| format!("failed to insert image row: {error}"))?;

            if inserted == 1 {
                report.newly_imported += 1;
            }

            let image_id: i64 = conn
                .query_row(
                    "SELECT id FROM images WHERE file_path = ?1",
                    params![canonical_str],
                    |row| row.get(0),
                )
                .map_err(|error| format!("failed to load image id: {error}"))?;

            conn.execute(
                "INSERT OR IGNORE INTO edits (image_id, edit_params_json, updated_at)
                 VALUES (?1, ?2, ?3)",
                params![image_id, default_edit, now],
            )
            .map_err(|error| format!("failed to insert default edits: {error}"))?;

            let thumb_path = thumbnail_path(cache_root, image_id);
            let (thumb_width, thumb_height) =
                ensure_jpeg_thumbnail(file_path, Path::new(&thumb_path))?;

            queries::upsert_thumbnail(
                &conn,
                image_id,
                &thumb_path,
                i64::from(thumb_width),
                i64::from(thumb_height),
                &now,
            )
            .map_err(|error| format!("failed to upsert thumbnail row: {error}"))?;
        }

        Ok(report)
    }

    pub fn list_images(&self) -> Result<Vec<ImageRecord>, String> {
        let conn = self.open_connection()?;
        queries::list_images(&conn).map_err(|error| format!("failed to list images: {error}"))
    }

    fn open_connection(&self) -> Result<Connection, String> {
        Connection::open(&self.path)
            .map_err(|error| format!("failed to open sqlite connection: {error}"))
    }
}

fn ensure_jpeg_thumbnail(source_path: &Path, thumb_path: &Path) -> Result<(u32, u32), String> {
    if thumb_path.exists() {
        let existing = ImageReader::open(thumb_path)
            .map_err(|error| format!("failed to open thumbnail {:?}: {error}", thumb_path))?
            .with_guessed_format()
            .map_err(|error| {
                format!(
                    "failed to detect thumbnail format {:?}: {error}",
                    thumb_path
                )
            })?
            .decode()
            .map_err(|error| format!("failed to decode thumbnail {:?}: {error}", thumb_path))?;
        return Ok((existing.width(), existing.height()));
    }

    let image = ImageReader::open(source_path)
        .map_err(|error| format!("failed to open source image {:?}: {error}", source_path))?
        .with_guessed_format()
        .map_err(|error| format!("failed to detect source format {:?}: {error}", source_path))?
        .decode()
        .map_err(|error| format!("failed to decode source image {:?}: {error}", source_path))?;

    let thumb = image.thumbnail(256, 256);
    if let Some(parent) = thumb_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create thumbnail directory: {error}"))?;
    }

    thumb
        .save_with_format(thumb_path, ImageFormat::Jpeg)
        .map_err(|error| format!("failed to write thumbnail {:?}: {error}", thumb_path))?;

    Ok((thumb.width(), thumb.height()))
}

fn now_iso_like() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    secs.to_string()
}

fn default_edit_params_json() -> Result<String, String> {
    serde_json::to_string(&json!({
        "exposure": 0.0,
        "contrast": 0.0,
        "temperature": 0.0,
        "tint": 0.0,
        "highlights": 0.0,
        "shadows": 0.0
    }))
    .map_err(|error| format!("failed to serialize default edit params: {error}"))
}

fn is_supported_jpeg(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            ext == "jpg" || ext == "jpeg"
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use tempfile::TempDir;

    #[test]
    fn initialize_creates_schema() {
        let dir = TempDir::new().expect("tempdir should be created");
        let db_path = dir.path().join("catalog.sqlite3");

        let db = CatalogDb::new(db_path.to_string_lossy().to_string());
        db.initialize().expect("schema should initialize");

        let conn = Connection::open(db_path).expect("db should open");
        let image_table_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='images'",
                [],
                |row| row.get(0),
            )
            .expect("query should succeed");

        assert_eq!(image_table_exists, 1);
    }

    #[test]
    fn import_jpegs_ignores_non_supported_files_and_creates_thumbnails() {
        let dir = TempDir::new().expect("tempdir should be created");
        let db_path = dir.path().join("catalog.sqlite3");
        let import_dir = dir.path().join("imports");
        let cache_dir = dir.path().join("cache");

        fs::create_dir_all(&import_dir).expect("import dir should exist");
        write_test_jpeg(&import_dir.join("one.jpg"), 640, 360);
        write_test_jpeg(&import_dir.join("two.jpeg"), 320, 320);
        fs::write(import_dir.join("skip.txt"), b"txt").expect("file should be written");

        let db = CatalogDb::new(db_path.to_string_lossy().to_string());
        db.initialize().expect("schema should initialize");

        let report = db
            .import_jpegs_from_folder(&import_dir.to_string_lossy(), &cache_dir.to_string_lossy())
            .expect("import should succeed");

        assert_eq!(report.supported_files, 2);
        assert_eq!(report.newly_imported, 2);

        let images = db.list_images().expect("list images should succeed");
        assert_eq!(images.len(), 2);

        let conn = Connection::open(db_path).expect("db should open");
        let thumb_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM thumbnails", [], |row| row.get(0))
            .expect("query should succeed");
        assert_eq!(thumb_rows, 2);

        let mut stmt = conn
            .prepare("SELECT file_path, width, height FROM thumbnails ORDER BY image_id")
            .expect("statement should prepare");
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .expect("query should work");

        for row in rows {
            let (file_path, width, height) = row.expect("row should decode");
            assert!(Path::new(&file_path).exists());
            assert!(width > 0 && width <= 256);
            assert!(height > 0 && height <= 256);
        }
    }

    fn write_test_jpeg(path: &Path, width: u32, height: u32) {
        let img = ImageBuffer::from_fn(width, height, |_x, _y| Rgb([120_u8, 40_u8, 200_u8]));
        img.save(path).expect("jpeg should be written");
    }
}
