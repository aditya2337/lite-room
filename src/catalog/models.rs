use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: i64,
    pub file_path: String,
    pub import_date: String,
    pub capture_date: Option<String>,
    pub rating: i64,
    pub flag: i64,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub scanned_files: usize,
    pub supported_files: usize,
    pub newly_imported: usize,
}
