mod config;
mod logging;
mod ui;

use std::process::ExitCode;

use config::AppConfig;
use lite_room_adapters::{
    present_decoded, present_image_row, FsThumbnailGenerator, ImageCrateDecoder,
    SqliteCatalogRepository, SystemClock, WalkdirFileScanner,
};
use lite_room_application::{
    ApplicationService, BootstrapCatalogCommand, ImportFolderCommand, ListImagesCommand,
    OpenImageCommand,
};
use lite_room_domain::ImageId;

fn main() -> ExitCode {
    logging::init_logging();
    let args: Vec<String> = std::env::args().collect();
    let config = AppConfig::default();

    let service = build_application_service(&config);
    if let Err(error) = service.bootstrap_catalog(BootstrapCatalogCommand) {
        eprintln!("failed to bootstrap lite-room: {error}");
        return ExitCode::from(1);
    }

    let command = parse_command(&args);
    match run_command(command, &service, &config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CommandError::Usage(msg)) => {
            eprintln!("{msg}");
            print_usage();
            ExitCode::from(2)
        }
        Err(CommandError::Runtime(msg)) => {
            eprintln!("{msg}");
            ExitCode::from(1)
        }
    }
}

fn build_application_service(config: &AppConfig) -> ApplicationService {
    ApplicationService::new(
        Box::new(SqliteCatalogRepository::new(config.catalog_path.clone())),
        Box::new(WalkdirFileScanner),
        Box::new(FsThumbnailGenerator),
        Box::new(ImageCrateDecoder),
        Box::new(SystemClock),
    )
}

#[derive(Debug, Clone)]
enum Command {
    Ui,
    Import { folder: String },
    List,
    Open { image_id: i64 },
}

#[derive(Debug, Clone)]
enum CommandError {
    Usage(String),
    Runtime(String),
}

fn parse_command(args: &[String]) -> Result<Command, CommandError> {
    if args.len() <= 1 {
        return Ok(Command::Ui);
    }

    match args[1].as_str() {
        "ui" => Ok(Command::Ui),
        "import" => {
            if args.len() < 3 {
                return Err(CommandError::Usage("missing folder path".to_string()));
            }
            Ok(Command::Import {
                folder: args[2].clone(),
            })
        }
        "list" => Ok(Command::List),
        "open" => {
            if args.len() < 3 {
                return Err(CommandError::Usage("missing image id".to_string()));
            }
            let image_id = args[2]
                .parse::<i64>()
                .map_err(|_| CommandError::Usage(format!("invalid image id: {}", args[2])))?;
            Ok(Command::Open { image_id })
        }
        other => Err(CommandError::Usage(format!("unknown command: {other}"))),
    }
}

fn run_command(
    command: Result<Command, CommandError>,
    service: &ApplicationService,
    config: &AppConfig,
) -> Result<(), CommandError> {
    match command? {
        Command::Ui => {
            let image_count = service
                .list_images(ListImagesCommand)
                .map_err(|error| CommandError::Runtime(error.to_string()))?
                .len();
            ui::launch_window(&config.catalog_path, &config.cache_dir, image_count)
                .map_err(CommandError::Runtime)
        }
        Command::Import { folder } => {
            let report = service
                .import_folder(ImportFolderCommand {
                    folder,
                    cache_root: config.cache_dir.clone(),
                })
                .map_err(|error| CommandError::Runtime(format!("import failed: {error}")))?;
            println!(
                "import finished: scanned={}, supported={}, newly_imported={}",
                report.scanned_files, report.supported_files, report.newly_imported
            );
            Ok(())
        }
        Command::List => {
            let images = service
                .list_images(ListImagesCommand)
                .map_err(|error| CommandError::Runtime(format!("list failed: {error}")))?;
            if images.is_empty() {
                println!("no images in catalog");
                return Ok(());
            }
            for image in images {
                println!("{}", present_image_row(&image));
            }
            Ok(())
        }
        Command::Open { image_id } => {
            let image_id = ImageId::new(image_id)
                .map_err(|error| CommandError::Usage(format!("invalid image id: {error}")))?;
            let decoded = service
                .open_image(OpenImageCommand { image_id })
                .map_err(|error| CommandError::Runtime(format!("open failed: {error}")))?;
            println!("{}", present_decoded(image_id.get(), &decoded));
            Ok(())
        }
    }
}

fn print_usage() {
    println!("usage:");
    println!("  lite-room ui");
    println!("  lite-room import <folder>");
    println!("  lite-room list");
    println!("  lite-room open <image_id>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_import_command() {
        let args = vec![
            "lite-room".to_string(),
            "import".to_string(),
            "photos".to_string(),
        ];
        let command = parse_command(&args).expect("import should parse");
        assert!(matches!(command, Command::Import { .. }));
    }

    #[test]
    fn parse_open_rejects_invalid_id() {
        let args = vec![
            "lite-room".to_string(),
            "open".to_string(),
            "abc".to_string(),
        ];
        let command = parse_command(&args);
        assert!(matches!(command, Err(CommandError::Usage(_))));
    }
}
