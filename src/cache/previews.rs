pub fn preview_path(cache_root: &str, image_id: i64) -> String {
    format!("{cache_root}/previews/{image_id}.jpg")
}
