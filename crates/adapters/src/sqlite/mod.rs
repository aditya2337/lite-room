mod queries;

use std::fs;
use std::path::PathBuf;

use lite_room_application::{ApplicationError, CatalogRepository, NewImage, UpsertImageResult};
use lite_room_domain::{ImageId, ImageRecord};
use rusqlite::{params, Connection};

use crate::migrations::MIGRATIONS;

#[derive(Debug, Clone)]
pub struct SqliteCatalogRepository {
    path: PathBuf,
}

impl SqliteCatalogRepository {
    pub fn new(path: String) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    fn open_connection(&self) -> Result<Connection, ApplicationError> {
        Connection::open(&self.path)
            .map_err(|error| ApplicationError::Persistence(error.to_string()))
    }
}

impl CatalogRepository for SqliteCatalogRepository {
    fn initialize(&self) -> Result<(), ApplicationError> {
        if self.path.as_os_str().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "catalog path must not be empty".to_string(),
            ));
        }

        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| ApplicationError::Io(error.to_string()))?;
            }
        }

        let conn = self.open_connection()?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL;")
            .map_err(|error| ApplicationError::Persistence(error.to_string()))?;

        for migration in MIGRATIONS {
            conn.execute_batch(migration)
                .map_err(|error| ApplicationError::Persistence(error.to_string()))?;
        }

        Ok(())
    }

    fn upsert_image(&self, image: &NewImage) -> Result<UpsertImageResult, ApplicationError> {
        let conn = self.open_connection()?;
        let inserted = conn
            .execute(
                "INSERT OR IGNORE INTO images
                 (file_path, import_date, capture_date, camera_model, iso, rating, flag, metadata_json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    image.file_path,
                    image.import_date,
                    image.capture_date,
                    image.camera_model,
                    image.iso,
                    image.rating,
                    image.flag,
                    image.metadata_json,
                ],
            )
            .map_err(|error| ApplicationError::Persistence(error.to_string()))?;

        let image_id_value: i64 = conn
            .query_row(
                "SELECT id FROM images WHERE file_path = ?1",
                params![image.file_path],
                |row| row.get(0),
            )
            .map_err(|error| ApplicationError::Persistence(error.to_string()))?;

        let image_id = ImageId::new(image_id_value)?;
        Ok(UpsertImageResult {
            image_id,
            inserted: inserted == 1,
        })
    }

    fn ensure_default_edit(
        &self,
        image_id: ImageId,
        edit_params_json: &str,
        updated_at: &str,
    ) -> Result<(), ApplicationError> {
        let conn = self.open_connection()?;
        conn.execute(
            "INSERT OR IGNORE INTO edits (image_id, edit_params_json, updated_at)
             VALUES (?1, ?2, ?3)",
            params![image_id.get(), edit_params_json, updated_at],
        )
        .map_err(|error| ApplicationError::Persistence(error.to_string()))?;
        Ok(())
    }

    fn upsert_thumbnail(
        &self,
        image_id: ImageId,
        file_path: &str,
        width: i64,
        height: i64,
        updated_at: &str,
    ) -> Result<(), ApplicationError> {
        let conn = self.open_connection()?;
        queries::upsert_thumbnail(&conn, image_id.get(), file_path, width, height, updated_at)
            .map_err(|error| ApplicationError::Persistence(error.to_string()))
    }

    fn list_images(&self) -> Result<Vec<ImageRecord>, ApplicationError> {
        let conn = self.open_connection()?;
        queries::list_images(&conn)
            .map_err(|error| ApplicationError::Persistence(error.to_string()))
    }

    fn find_image_by_id(&self, image_id: ImageId) -> Result<Option<ImageRecord>, ApplicationError> {
        let conn = self.open_connection()?;
        queries::find_image_by_id(&conn, image_id.get())
            .map_err(|error| ApplicationError::Persistence(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn initialize_creates_schema() {
        let dir = TempDir::new().expect("tempdir");
        let db_path = dir.path().join("catalog.sqlite3");
        let repo = SqliteCatalogRepository::new(db_path.to_string_lossy().to_string());
        repo.initialize().expect("initialize");

        let conn = Connection::open(db_path).expect("open");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='images'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(count, 1);
    }
}
