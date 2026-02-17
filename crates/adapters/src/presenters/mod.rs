use lite_room_domain::{DecodedImage, ImageRecord};

pub fn present_image_row(image: &ImageRecord) -> String {
    format!(
        "{}\t{}\t{}\t{}",
        image.id.get(),
        image_kind_from_path(&image.file_path),
        image.import_date,
        image.file_path
    )
}

pub fn present_decoded(image_id: i64, decoded: &DecodedImage) -> String {
    format!(
        "opened image {} (kind={:?}, {}x{})",
        image_id, decoded.kind, decoded.width, decoded.height
    )
}

fn image_kind_from_path(path: &str) -> &'static str {
    use std::path::Path;
    match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
    {
        Some(ext) if ext == "jpg" || ext == "jpeg" => "JPEG",
        Some(ext) if ext == "cr2" || ext == "nef" || ext == "arw" || ext == "dng" => "RAW",
        _ => "UNKNOWN",
    }
}
