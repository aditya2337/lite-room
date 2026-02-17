use lite_room_domain::{ImageId, ImageRecord};
use rusqlite::{params, Connection, Result};

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
        let id_value: i64 = row.get(0)?;
        Ok(ImageRecord {
            id: ImageId::new(id_value).expect("database returned non-positive image id"),
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

pub fn find_image_by_id(conn: &Connection, image_id: i64) -> Result<Option<ImageRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, import_date, capture_date, rating, flag, metadata_json
         FROM images
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![image_id])?;
    if let Some(row) = rows.next()? {
        let id_value: i64 = row.get(0)?;
        return Ok(Some(ImageRecord {
            id: ImageId::new(id_value).expect("database returned non-positive image id"),
            file_path: row.get(1)?,
            import_date: row.get(2)?,
            capture_date: row.get(3)?,
            rating: row.get(4)?,
            flag: row.get(5)?,
            metadata_json: row.get(6)?,
        }));
    }

    Ok(None)
}
