use std::fs;
use std::path::Path;

use image::{io::Reader as ImageReader, ImageBuffer, ImageFormat, Rgb};
use lite_room_application::{ApplicationError, ThumbnailArtifact, ThumbnailGenerator};
use lite_room_domain::{detect_image_kind, ImageId, ImageKind};

#[derive(Debug, Default)]
pub struct FsThumbnailGenerator;

impl ThumbnailGenerator for FsThumbnailGenerator {
    fn ensure_thumbnail(
        &self,
        source_path: &Path,
        cache_root: &str,
        image_id: ImageId,
    ) -> Result<ThumbnailArtifact, ApplicationError> {
        let thumb_path = format!("{cache_root}/thumbs/{}.jpg", image_id.get());
        let thumb_path_ref = Path::new(&thumb_path);

        let (width, height) = match detect_image_kind(source_path) {
            ImageKind::Jpeg => ensure_jpeg_thumbnail(source_path, thumb_path_ref)?,
            ImageKind::Raw | ImageKind::Unsupported => {
                ensure_placeholder_thumbnail(thumb_path_ref)?
            }
        };

        Ok(ThumbnailArtifact {
            file_path: thumb_path,
            width,
            height,
        })
    }
}

fn ensure_jpeg_thumbnail(
    source_path: &Path,
    thumb_path: &Path,
) -> Result<(u32, u32), ApplicationError> {
    if thumb_path.exists() {
        let existing = ImageReader::open(thumb_path)
            .map_err(|error| ApplicationError::Io(error.to_string()))?
            .with_guessed_format()
            .map_err(|error| ApplicationError::Decode(error.to_string()))?
            .decode()
            .map_err(|error| ApplicationError::Decode(error.to_string()))?;
        return Ok((existing.width(), existing.height()));
    }

    let image = ImageReader::open(source_path)
        .map_err(|error| ApplicationError::Io(error.to_string()))?
        .with_guessed_format()
        .map_err(|error| ApplicationError::Decode(error.to_string()))?
        .decode()
        .map_err(|error| ApplicationError::Decode(error.to_string()))?;

    let thumb = image.thumbnail(256, 256);
    if let Some(parent) = thumb_path.parent() {
        fs::create_dir_all(parent).map_err(|error| ApplicationError::Io(error.to_string()))?;
    }

    thumb
        .save_with_format(thumb_path, ImageFormat::Jpeg)
        .map_err(|error| ApplicationError::Io(error.to_string()))?;

    Ok((thumb.width(), thumb.height()))
}

fn ensure_placeholder_thumbnail(thumb_path: &Path) -> Result<(u32, u32), ApplicationError> {
    if let Some(parent) = thumb_path.parent() {
        fs::create_dir_all(parent).map_err(|error| ApplicationError::Io(error.to_string()))?;
    }

    if thumb_path.exists() {
        let existing = ImageReader::open(thumb_path)
            .map_err(|error| ApplicationError::Io(error.to_string()))?
            .with_guessed_format()
            .map_err(|error| ApplicationError::Decode(error.to_string()))?
            .decode()
            .map_err(|error| ApplicationError::Decode(error.to_string()))?;
        return Ok((existing.width(), existing.height()));
    }

    let placeholder = ImageBuffer::from_fn(256, 256, |_x, _y| Rgb([48_u8, 48_u8, 48_u8]));
    placeholder
        .save_with_format(thumb_path, ImageFormat::Jpeg)
        .map_err(|error| ApplicationError::Io(error.to_string()))?;

    Ok((256, 256))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use tempfile::TempDir;

    #[test]
    fn creates_thumbnail_for_jpeg() {
        let dir = TempDir::new().expect("tempdir");
        let src = dir.path().join("x.jpg");
        let img = ImageBuffer::from_fn(500, 300, |_x, _y| Rgb([10_u8, 20_u8, 30_u8]));
        img.save(&src).expect("save");

        let generator = FsThumbnailGenerator;
        let out = generator
            .ensure_thumbnail(
                &src,
                &dir.path().to_string_lossy(),
                ImageId::new(1).expect("id"),
            )
            .expect("thumbnail");

        assert_eq!(out.width, 256);
        assert_eq!(out.height, 154);
    }
}
