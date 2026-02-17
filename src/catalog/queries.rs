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

pub fn find_image_by_id(conn: &Connection, image_id: i64) -> Result<Option<ImageRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, file_path, import_date, capture_date, rating, flag, metadata_json
         FROM images
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![image_id])?;
    if let Some(row) = rows.next()? {
        return Ok(Some(ImageRecord {
            id: row.get(0)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory sqlite should open");
        conn.execute_batch(
            "
            CREATE TABLE images (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL UNIQUE,
                import_date TEXT NOT NULL,
                capture_date TEXT,
                camera_model TEXT,
                iso INTEGER,
                rating INTEGER NOT NULL DEFAULT 0,
                flag INTEGER NOT NULL DEFAULT 0,
                metadata_json TEXT NOT NULL
            );

            CREATE TABLE thumbnails (
                image_id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )
        .expect("schema should be created");
        conn
    }

    #[test]
    fn upsert_thumbnail_inserts_then_updates() {
        let conn = setup_conn();

        upsert_thumbnail(&conn, 7, "cache/thumbs/7.jpg", 256, 256, "100")
            .expect("first upsert should insert");
        upsert_thumbnail(&conn, 7, "cache/thumbs/7-new.jpg", 512, 512, "101")
            .expect("second upsert should update");

        let row: (String, i64, i64, String) = conn
            .query_row(
                "SELECT file_path, width, height, updated_at FROM thumbnails WHERE image_id = 7",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("thumbnail row should exist");

        assert_eq!(row.0, "cache/thumbs/7-new.jpg");
        assert_eq!(row.1, 512);
        assert_eq!(row.2, 512);
        assert_eq!(row.3, "101");
    }

    #[test]
    fn list_images_orders_by_capture_or_import_desc_then_id_desc() {
        let conn = setup_conn();

        conn.execute(
            "INSERT INTO images (id, file_path, import_date, capture_date, rating, flag, metadata_json)
             VALUES (?1, ?2, ?3, ?4, 0, 0, '{}')",
            params![1_i64, "a.jpg", "100", Option::<String>::None],
        )
        .expect("row insert should succeed");
        conn.execute(
            "INSERT INTO images (id, file_path, import_date, capture_date, rating, flag, metadata_json)
             VALUES (?1, ?2, ?3, ?4, 0, 0, '{}')",
            params![2_i64, "b.jpg", "200", Option::<String>::None],
        )
        .expect("row insert should succeed");
        conn.execute(
            "INSERT INTO images (id, file_path, import_date, capture_date, rating, flag, metadata_json)
             VALUES (?1, ?2, ?3, ?4, 0, 0, '{}')",
            params![3_i64, "c.jpg", "150", Some("300".to_string())],
        )
        .expect("row insert should succeed");
        conn.execute(
            "INSERT INTO images (id, file_path, import_date, capture_date, rating, flag, metadata_json)
             VALUES (?1, ?2, ?3, ?4, 0, 0, '{}')",
            params![4_i64, "d.jpg", "250", Some("300".to_string())],
        )
        .expect("row insert should succeed");

        let images = list_images(&conn).expect("query should succeed");
        let ids: Vec<i64> = images.into_iter().map(|image| image.id).collect();

        assert_eq!(ids, vec![4, 3, 2, 1]);
    }

    #[test]
    fn find_image_by_id_returns_matching_row() {
        let conn = setup_conn();
        conn.execute(
            "INSERT INTO images (id, file_path, import_date, capture_date, rating, flag, metadata_json)
             VALUES (?1, ?2, ?3, NULL, 0, 0, '{}')",
            params![11_i64, "x.jpg", "100"],
        )
        .expect("row insert should succeed");

        let found = find_image_by_id(&conn, 11)
            .expect("query should succeed")
            .expect("row should exist");
        assert_eq!(found.id, 11);
        assert_eq!(found.file_path, "x.jpg");

        let missing = find_image_by_id(&conn, 999).expect("query should succeed");
        assert!(missing.is_none());
    }
}
