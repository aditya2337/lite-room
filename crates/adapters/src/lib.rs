pub mod fs;
pub mod migrations;
pub mod presenters;
pub mod sqlite;

pub use fs::{FsThumbnailGenerator, SystemClock, WalkdirFileScanner};
pub use presenters::{present_decoded, present_edit_params, present_image_row};
pub use sqlite::SqliteCatalogRepository;

use lite_room_application::ApplicationError;
use lite_room_application::ImageDecoder;
use lite_room_domain::{detect_image_kind, DecodedImage, ImageKind};
use std::path::Path;

#[derive(Debug, Default)]
pub struct ImageCrateDecoder;

impl ImageDecoder for ImageCrateDecoder {
    fn decode_for_preview(&self, path: &Path) -> Result<DecodedImage, ApplicationError> {
        match detect_image_kind(path) {
            ImageKind::Jpeg => {
                let image = image::io::Reader::open(path)
                    .map_err(|error| ApplicationError::Decode(error.to_string()))?
                    .with_guessed_format()
                    .map_err(|error| ApplicationError::Decode(error.to_string()))?
                    .decode()
                    .map_err(|error| ApplicationError::Decode(error.to_string()))?;

                Ok(DecodedImage {
                    width: image.width(),
                    height: image.height(),
                    kind: ImageKind::Jpeg,
                })
            }
            ImageKind::Raw => Err(ApplicationError::Decode(format!(
                "RAW decode not implemented yet for {:?}",
                path
            ))),
            ImageKind::Unsupported => Err(ApplicationError::Decode(format!(
                "unsupported image format: {:?}",
                path
            ))),
        }
    }
}
