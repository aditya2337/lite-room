use std::path::Path;

use image::ImageReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Jpeg,
    Raw,
    Unsupported,
}

#[derive(Debug, Clone)]
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

pub fn decode_for_preview(path: &Path) -> Result<DecodedImage, String> {
    match detect_image_kind(path) {
        ImageKind::Jpeg => {
            let image = ImageReader::open(path)
                .map_err(|error| format!("failed to open JPEG {:?}: {error}", path))?
                .with_guessed_format()
                .map_err(|error| format!("failed to detect JPEG format {:?}: {error}", path))?
                .decode()
                .map_err(|error| format!("failed to decode JPEG {:?}: {error}", path))?;

            Ok(DecodedImage {
                width: image.width(),
                height: image.height(),
                kind: ImageKind::Jpeg,
            })
        }
        ImageKind::Raw => Err(format!(
            "RAW decode not implemented yet for {:?}. Phase 2 libraw integration pending.",
            path
        )),
        ImageKind::Unsupported => Err(format!("unsupported image format: {:?}", path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use tempfile::TempDir;

    #[test]
    fn detect_image_kind_handles_supported_formats() {
        assert_eq!(detect_image_kind(Path::new("a.jpg")), ImageKind::Jpeg);
        assert_eq!(detect_image_kind(Path::new("a.jpeg")), ImageKind::Jpeg);
        assert_eq!(detect_image_kind(Path::new("a.cr2")), ImageKind::Raw);
        assert_eq!(detect_image_kind(Path::new("a.nef")), ImageKind::Raw);
        assert_eq!(detect_image_kind(Path::new("a.arw")), ImageKind::Raw);
        assert_eq!(detect_image_kind(Path::new("a.dng")), ImageKind::Raw);
        assert_eq!(
            detect_image_kind(Path::new("a.png")),
            ImageKind::Unsupported
        );
    }

    #[test]
    fn decode_for_preview_returns_dimensions_for_jpeg() {
        let dir = TempDir::new().expect("tempdir should be created");
        let path = dir.path().join("sample.jpg");
        let img = ImageBuffer::from_fn(800, 450, |_x, _y| Rgb([10_u8, 20_u8, 30_u8]));
        img.save(&path).expect("jpeg should be saved");

        let decoded = decode_for_preview(&path).expect("jpeg should decode");
        assert_eq!(decoded.width, 800);
        assert_eq!(decoded.height, 450);
        assert_eq!(decoded.kind, ImageKind::Jpeg);
    }
}
