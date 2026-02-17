use crate::app::events::AppEvent;
use crate::app::state::AppState;
use crate::catalog::db::CatalogDb;
use crate::catalog::models::{ImageRecord, ImportReport};
use crate::infra::config::AppConfig;

pub struct ApplicationController {
    config: AppConfig,
    state: AppState,
    catalog: CatalogDb,
}

impl ApplicationController {
    pub fn new(config: AppConfig) -> Self {
        Self {
            catalog: CatalogDb::new(config.catalog_path.clone()),
            config,
            state: AppState::default(),
        }
    }

    pub fn bootstrap(&mut self) -> Result<(), String> {
        self.catalog.initialize()?;
        self.state.catalog_loaded = true;
        Ok(())
    }

    pub fn dispatch(&mut self, event: AppEvent) {
        match event {
            AppEvent::ImportFolder(path) => {
                if let Err(error) = self.import_folder(&path) {
                    eprintln!("import failed: {error}");
                }
            }
            AppEvent::OpenImage(image_id) => {
                self.state.selected_image_id = Some(image_id);
            }
            AppEvent::UpdateExposure(_value) => {
                // Edit pipeline lands in phase 3.
            }
            AppEvent::Quit => {}
        }
    }

    pub fn run(&mut self) {
        println!(
            "lite-room initialized (catalog: {}, cache: {})",
            self.config.catalog_path, self.config.cache_dir
        );
        self.dispatch(AppEvent::Quit);
    }

    pub fn import_folder(&mut self, path: &str) -> Result<ImportReport, String> {
        let report = self
            .catalog
            .import_jpegs_from_folder(path, &self.config.cache_dir)?;
        self.state.last_imported = report.newly_imported;
        Ok(report)
    }

    pub fn list_images(&self) -> Result<Vec<ImageRecord>, String> {
        self.catalog.list_images()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn bootstrap_marks_catalog_loaded() {
        let mut controller = ApplicationController::new(AppConfig::default());
        controller.bootstrap().expect("bootstrap should succeed");
        assert!(controller.state.catalog_loaded);
    }

    #[test]
    fn dispatch_open_image_sets_selected_image_id() {
        let mut controller = ApplicationController::new(AppConfig::default());

        controller.dispatch(AppEvent::OpenImage(42));

        assert_eq!(controller.state.selected_image_id, Some(42));
    }

    #[test]
    fn dispatch_import_folder_updates_last_imported_count() {
        let dir = TempDir::new().expect("tempdir should be created");
        let catalog_path = dir.path().join("catalog.sqlite3");
        let import_dir = dir.path().join("imports");
        let cache_dir = dir.path().join("cache");

        fs::create_dir_all(&import_dir).expect("import dir should exist");
        write_test_jpeg(&import_dir.join("image.jpg"), 500, 300);

        let config = AppConfig {
            catalog_path: catalog_path.to_string_lossy().to_string(),
            cache_dir: cache_dir.to_string_lossy().to_string(),
        };
        let mut controller = ApplicationController::new(config);
        controller.bootstrap().expect("bootstrap should succeed");

        controller.dispatch(AppEvent::ImportFolder(
            import_dir.to_string_lossy().to_string(),
        ));

        assert_eq!(controller.state.last_imported, 1);
    }

    fn write_test_jpeg(path: &Path, width: u32, height: u32) {
        let img = ImageBuffer::from_fn(width, height, |_x, _y| Rgb([120_u8, 40_u8, 200_u8]));
        img.save(path).expect("jpeg should be written");
    }
}
