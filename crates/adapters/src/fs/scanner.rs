use std::path::Path;

use lite_room_application::{ApplicationError, FileScanSummary, FileScanner, ScannedFile};
use lite_room_domain::{detect_image_kind, ImageKind};
use walkdir::WalkDir;

#[derive(Debug, Default)]
pub struct WalkdirFileScanner;

impl FileScanner for WalkdirFileScanner {
    fn scan_supported(&self, folder: &str) -> Result<FileScanSummary, ApplicationError> {
        let folder_path = Path::new(folder);
        if !folder_path.is_dir() {
            return Err(ApplicationError::InvalidInput(format!(
                "folder does not exist or is not a directory: {folder}"
            )));
        }

        let mut summary = FileScanSummary::default();

        for entry in WalkDir::new(folder_path).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            summary.scanned_files += 1;
            let file_path = entry.path();
            let image_kind = detect_image_kind(file_path);
            if image_kind == ImageKind::Unsupported {
                continue;
            }

            let canonical = file_path
                .canonicalize()
                .map_err(|error| ApplicationError::Io(error.to_string()))?;
            let metadata = file_path
                .metadata()
                .map_err(|error| ApplicationError::Io(error.to_string()))?;
            let extension = file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();

            summary.supported_files += 1;
            summary.files.push(ScannedFile {
                canonical_path: canonical,
                extension,
                file_size: metadata.len(),
                image_kind,
            });
        }

        Ok(summary)
    }
}
