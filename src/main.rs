#![allow(dead_code)]

mod app;
mod cache;
mod catalog;
mod engine;
mod infra;
mod ui;

use app::controller::ApplicationController;
use infra::config::AppConfig;

fn main() {
    let config = AppConfig::default();
    let mut controller = ApplicationController::new(config);

    if let Err(error) = controller.bootstrap() {
        eprintln!("failed to bootstrap lite-room: {error}");
        std::process::exit(1);
    }

    controller.run();
}
