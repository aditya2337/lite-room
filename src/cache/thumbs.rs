pub fn thumbnail_path(cache_root: &str, image_id: i64) -> String {
    format!("{cache_root}/thumbs/{image_id}.jpg")
}
