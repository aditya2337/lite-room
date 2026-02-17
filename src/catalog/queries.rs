use rusqlite::{params, Connection, Result};

use crate::catalog::models::ImageRecord;

pub fn upsert_thumbnail(
    conn: &Connection,
    image_id: i64,
    file_path: &str,
    width: i64,
    height: i64,
    updated_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO thumbnails (image_id, file_path, width, height, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(image_id) DO UPDATE SET
            file_path = excluded.file_path,
            width = excluded.width,
            height = excluded.height,
            updated_at = excluded.updated_at",
        params![image_id, file_path, width, height, updated_at],
    )?;

    Ok(())
}

pub fn list_images(conn: &Connection) -> Result<Vec<ImageRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, import_date, capture_date, rating, flag, metadata_json
         FROM images
         ORDER BY COALESCE(capture_date, import_date) DESC, id DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            file_path: row.get(1)?,
            import_date: row.get(2)?,
            capture_date: row.get(3)?,
            rating: row.get(4)?,
            flag: row.get(5)?,
            metadata_json: row.get(6)?,
        })
    })?;

    rows.collect()
}
