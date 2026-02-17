use std::path::{Path, PathBuf};

use lite_room_domain::{ImageId, ImageKind, ImageRecord};

use crate::ApplicationError;

#[derive(Debug, Clone)]
pub struct NewImage {
    pub file_path: String,
    pub import_date: String,
    pub capture_date: Option<String>,
    pub camera_model: Option<String>,
    pub iso: Option<i64>,
    pub rating: i64,
    pub flag: i64,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Copy)]
pub struct UpsertImageResult {
    pub image_id: ImageId,
    pub inserted: bool,
}

pub trait CatalogRepository {
    fn initialize(&self) -> Result<(), ApplicationError>;

    fn upsert_image(&self, image: &NewImage) -> Result<UpsertImageResult, ApplicationError>;

    fn ensure_default_edit(
        &self,
        image_id: ImageId,
        edit_params_json: &str,
        updated_at: &str,
    ) -> Result<(), ApplicationError>;

    fn upsert_thumbnail(
        &self,
        image_id: ImageId,
        file_path: &str,
        width: i64,
        height: i64,
        updated_at: &str,
    ) -> Result<(), ApplicationError>;

    fn list_images(&self) -> Result<Vec<ImageRecord>, ApplicationError>;

    fn find_image_by_id(&self, image_id: ImageId) -> Result<Option<ImageRecord>, ApplicationError>;
}

#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub canonical_path: PathBuf,
    pub extension: String,
    pub file_size: u64,
    pub image_kind: ImageKind,
}

#[derive(Debug, Clone, Default)]
pub struct FileScanSummary {
    pub scanned_files: usize,
    pub supported_files: usize,
    pub files: Vec<ScannedFile>,
}

pub trait FileScanner {
    fn scan_supported(&self, folder: &str) -> Result<FileScanSummary, ApplicationError>;
}

#[derive(Debug, Clone)]
pub struct ThumbnailArtifact {
    pub file_path: String,
    pub width: u32,
    pub height: u32,
}

pub trait ThumbnailGenerator {
    fn ensure_thumbnail(
        &self,
        source_path: &Path,
        cache_root: &str,
        image_id: ImageId,
    ) -> Result<ThumbnailArtifact, ApplicationError>;
}

pub trait ImageDecoder {
    fn decode_for_preview(
        &self,
        path: &Path,
    ) -> Result<lite_room_domain::DecodedImage, ApplicationError>;
}

pub trait Clock {
    fn now_timestamp_string(&self) -> String;
}
