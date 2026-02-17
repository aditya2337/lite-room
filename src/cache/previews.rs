pub fn preview_path(cache_root: &str, image_id: i64) -> String {
    format!("{cache_root}/previews/{image_id}.jpg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_path_uses_cache_root_and_id() {
        let path = preview_path("cache-dir", 15);
        assert_eq!(path, "cache-dir/previews/15.jpg");
    }
}
