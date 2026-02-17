pub fn thumbnail_path(cache_root: &str, image_id: i64) -> String {
    format!("{cache_root}/thumbs/{image_id}.jpg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_path_uses_cache_root_and_id() {
        let path = thumbnail_path("cache-dir", 15);
        assert_eq!(path, "cache-dir/thumbs/15.jpg");
    }
}
