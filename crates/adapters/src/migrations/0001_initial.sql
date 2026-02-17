CREATE TABLE IF NOT EXISTS images (
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

CREATE TABLE IF NOT EXISTS edits (
  image_id INTEGER PRIMARY KEY,
  edit_params_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(image_id) REFERENCES images(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS thumbnails (
  image_id INTEGER PRIMARY KEY,
  file_path TEXT NOT NULL,
  width INTEGER NOT NULL,
  height INTEGER NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY(image_id) REFERENCES images(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_images_import_date ON images(import_date);
CREATE INDEX IF NOT EXISTS idx_images_capture_date ON images(capture_date);
