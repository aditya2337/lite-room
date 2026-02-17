use eframe::egui;

pub struct AppShellState {
    pub catalog_path: String,
    pub cache_dir: String,
    pub image_count: usize,
}

impl AppShellState {
    fn new(catalog_path: String, cache_dir: String, image_count: usize) -> Self {
        Self {
            catalog_path,
            cache_dir,
            image_count,
        }
    }
}

impl eframe::App for AppShellState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.heading("lite-room");
            ui.label("Bare minimum app shell");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.separator();
            ui.label(format!("Catalog: {}", self.catalog_path));
            ui.label(format!("Cache: {}", self.cache_dir));
            ui.label(format!("Images in catalog: {}", self.image_count));
            ui.separator();
            ui.label("Next implementation steps:");
            ui.label("1. Grid view");
            ui.label("2. Single image edit view");
            ui.label("3. RAW decode via libraw");
        });
    }
}

pub fn launch_bare_window(
    catalog_path: &str,
    cache_dir: &str,
    image_count: usize,
) -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "lite-room",
        options,
        Box::new(|_cc| {
            Ok(Box::new(AppShellState::new(
                catalog_path.to_string(),
                cache_dir.to_string(),
                image_count,
            )))
        }),
    )
    .map_err(|error| format!("failed to start UI: {error}"))
}
