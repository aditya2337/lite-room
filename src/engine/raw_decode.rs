#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
}

pub fn decode_raw(_path: &str) -> Result<DecodedImage, String> {
    // Placeholder; libraw integration lands in phase 2.
    Ok(DecodedImage {
        width: 0,
        height: 0,
    })
}
