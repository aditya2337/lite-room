use crate::app::events::AppEvent;
use crate::app::state::AppState;
use crate::catalog::db::CatalogDb;
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
                match self
                    .catalog
                    .import_jpegs_from_folder(&path, &self.config.cache_dir)
                {
                    Ok(report) => {
                        self.state.last_imported = report.newly_imported;
                        println!(
                            "import finished: scanned={}, supported={}, newly_imported={}",
                            report.scanned_files, report.supported_files, report.newly_imported
                        );
                    }
                    Err(error) => {
                        eprintln!("import failed: {error}");
                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_marks_catalog_loaded() {
        let mut controller = ApplicationController::new(AppConfig::default());
        controller.bootstrap().expect("bootstrap should succeed");
        assert!(controller.state.catalog_loaded);
    }
}
