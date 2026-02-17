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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_local_catalog_and_cache_paths() {
        let config = AppConfig::default();
        assert_eq!(config.catalog_path, "catalog.sqlite3");
        assert_eq!(config.cache_dir, "cache");
    }
}
