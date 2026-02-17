#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub catalog_loaded: bool,
    pub selected_image_id: Option<i64>,
    pub last_imported: usize,
}
