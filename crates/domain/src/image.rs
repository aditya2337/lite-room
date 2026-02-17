use std::path::Path;

use crate::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageId(i64);

impl ImageId {
    pub fn new(value: i64) -> Result<Self, DomainError> {
        if value <= 0 {
            return Err(DomainError::InvalidImageId(value));
        }
        Ok(Self(value))
    }

    pub fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Raw,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageRecord {
    pub id: ImageId,
    pub file_path: String,
    pub import_date: String,
    pub capture_date: Option<String>,
    pub rating: i64,
    pub flag: i64,
    pub metadata_json: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub scanned_files: usize,
    pub supported_files: usize,
    pub newly_imported: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub kind: ImageKind,
}

pub fn detect_image_kind(path: &Path) -> ImageKind {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return ImageKind::Unsupported;
    };

    match ext.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => ImageKind::Jpeg,
        "cr2" | "nef" | "arw" | "dng" => ImageKind::Raw,
        _ => ImageKind::Unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_id_must_be_positive() {
        assert!(ImageId::new(1).is_ok());
        assert!(matches!(
            ImageId::new(0),
            Err(DomainError::InvalidImageId(0))
        ));
    }

    #[test]
    fn image_kind_detection_works() {
        assert_eq!(detect_image_kind(Path::new("a.jpg")), ImageKind::Jpeg);
        assert_eq!(detect_image_kind(Path::new("a.nef")), ImageKind::Raw);
        assert_eq!(
            detect_image_kind(Path::new("a.png")),
            ImageKind::Unsupported
        );
    }
}
