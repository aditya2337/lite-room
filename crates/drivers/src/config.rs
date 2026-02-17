#[derive(Debug, Clone)]
pub struct AppConfig {
    pub catalog_path: String,
    pub cache_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            catalog_path: "catalog.sqlite3".to_string(),
            cache_dir: "cache".to_string(),
        }
    }
}
