#[derive(Debug, Clone)]
pub enum AppEvent {
    ImportFolder(String),
    OpenImage(i64),
    UpdateExposure(f32),
    Quit,
}
