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
    let args: Vec<String> = std::env::args().collect();
    let config = AppConfig::default();
    let mut controller = ApplicationController::new(config);

    if let Err(error) = controller.bootstrap() {
        eprintln!("failed to bootstrap lite-room: {error}");
        std::process::exit(1);
    }

    if args.len() <= 1 {
        controller.run();
        print_usage();
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
                    println!("{}\t{}\t{}", image.id, image.import_date, image.file_path);
                }
            }
            Err(error) => {
                eprintln!("list failed: {error}");
                std::process::exit(1);
            }
        },
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}

fn print_usage() {
    println!("usage:");
    println!("  lite-room import <folder>");
    println!("  lite-room list");
}
