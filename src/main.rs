#![allow(dead_code)]

mod app;
mod cache;
mod catalog;
mod engine;
mod infra;
mod ui;

use app::controller::ApplicationController;
use infra::config::AppConfig;
use ui::app_shell::launch_bare_window;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = AppConfig::default();
    let mut controller = ApplicationController::new(config.clone());

    if let Err(error) = controller.bootstrap() {
        eprintln!("failed to bootstrap lite-room: {error}");
        std::process::exit(1);
    }

    if args.len() <= 1 {
        if let Err(error) = launch_ui(&controller, &config) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }

    match args[1].as_str() {
        "import" => {
            if args.len() < 3 {
                eprintln!("missing folder path");
                print_usage();
                std::process::exit(2);
            }

            let folder = &args[2];
            match controller.import_folder(folder) {
                Ok(report) => {
                    println!(
                        "import finished: scanned={}, supported={}, newly_imported={}",
                        report.scanned_files, report.supported_files, report.newly_imported
                    );
                }
                Err(error) => {
                    eprintln!("import failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        "list" => match controller.list_images() {
            Ok(images) => {
                if images.is_empty() {
                    println!("no images in catalog");
                    return;
                }

                for image in images {
                    let format = detect_format_from_path(&image.file_path);
                    println!(
                        "{}\t{}\t{}\t{}",
                        image.id, format, image.import_date, image.file_path
                    );
                }
            }
            Err(error) => {
                eprintln!("list failed: {error}");
                std::process::exit(1);
            }
        },
        "open" => {
            if args.len() < 3 {
                eprintln!("missing image id");
                print_usage();
                std::process::exit(2);
            }

            let image_id: i64 = match args[2].parse() {
                Ok(value) => value,
                Err(_) => {
                    eprintln!("invalid image id: {}", args[2]);
                    std::process::exit(2);
                }
            };

            match controller.open_image(image_id) {
                Ok(decoded) => {
                    println!(
                        "opened image {} (kind={:?}, {}x{})",
                        image_id, decoded.kind, decoded.width, decoded.height
                    );
                }
                Err(error) => {
                    eprintln!("open failed: {error}");
                    std::process::exit(1);
                }
            }
        }
        "ui" => {
            if let Err(error) = launch_ui(&controller, &config) {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}

fn detect_format_from_path(path: &str) -> &'static str {
    use std::path::Path;
    match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
    {
        Some(ext) if ext == "jpg" || ext == "jpeg" => "JPEG",
        Some(ext) if ext == "cr2" || ext == "nef" || ext == "arw" || ext == "dng" => "RAW",
        _ => "UNKNOWN",
    }
}

fn print_usage() {
    println!("usage:");
    println!("  lite-room ui");
    println!("  lite-room import <folder>");
    println!("  lite-room list");
    println!("  lite-room open <image_id>");
}

fn launch_ui(controller: &ApplicationController, config: &AppConfig) -> Result<(), String> {
    let image_count = controller.list_images()?.len();
    launch_bare_window(&config.catalog_path, &config.cache_dir, image_count)
}
